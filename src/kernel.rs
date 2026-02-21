#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate limine;

use core::arch::asm;
use core::fmt::{self, Write};
use limine::request::{
    FramebufferRequest, HhdmRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker,
    StackSizeRequest,
};
use limine::BaseRevision;
use limine::memory_map::EntryType;
use linked_list_allocator::LockedHeap;
use spleen_font::FONT_16X32;
use vibe_framebuffer::{Cursor, Font};

// --- Limine Requests ---

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(0x10000);

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

// --- Global Allocator ---

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static mut UI_CURSOR: Option<Cursor> = None;

pub fn _print(args: fmt::Arguments) {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            let _ = cursor.write_fmt(args);
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn init_heap(start: usize, size: usize) {
    unsafe {
        ALLOCATOR.lock().init(start as *mut u8, size);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("VIBE OS: Heap Allocation Error - Layout: {:?}", layout);
}

// --- Entry Point ---

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    // --- INITIAL DEBUG SETUP ---
    // Fetch FB early to draw bars. If this fails, we stay black.
    let fb_binding = FRAMEBUFFER_REQUEST.get_response();
    let fb_res = fb_binding.as_ref().expect("FB Failed");
    let fb = fb_res.framebuffers().next().expect("No FB found");
    let fb_addr = fb.addr() as *mut u32;
    let width = fb.width() as usize;

    // BAR 1: WHITE (Kernel Reached _start)
    unsafe {
        for i in 0..(width * 20) {
            core::ptr::write_volatile(fb_addr.add(i), 0xffffff);
        }
    }

    // --- HHDM STAGE ---
    let hhdm_binding = HHDM_REQUEST.get_response();
    let hhdm_offset = hhdm_binding.as_ref().expect("HHDM Failed").offset();

    // BAR 2: YELLOW (HHDM Offset Acquired)
    unsafe {
        for i in (width * 20)..(width * 40) {
            core::ptr::write_volatile(fb_addr.add(i), 0xffff00);
        }
    }
// --- STAGE: DYNAMIC HEAP SEARCH ---
    let mut heap_virt_addr: u64 = 0;
    let heap_size = 16 * 1024 * 1024; 

    for (i, entry) in memmap_response.entries().iter().enumerate() {
        // DRAW A TINY SQUARES FOR EACH ENTRY
        // Green = Usable, Red = Reserved/Other
        let color = if entry.entry_type == EntryType::USABLE { 0x9ece6a } else { 0xf7768e };
        unsafe {
            for x in (i * 30)..(i * 30 + 25) {
                for y in 100..120 { // Draw below your debug bars
                    core::ptr::write_volatile(fb_addr.add(y * width + x), color);
                }
            }
        }

        if entry.entry_type == EntryType::USABLE && entry.base >= 0x1000000 {
            if entry.length >= heap_size as u64 {
                // Check if this physical address is likely mapped (Stay low for now)
                if entry.base < 0x80000000 { // Stay below 2GB
                    heap_virt_addr = entry.base + hhdm_offset;
                    // Don't break yet, let's draw all the squares first
                }
            }
        }
    }

    // BAR 3: BLUE (Memory Chunk Found)
    unsafe {
        for i in (width * 40)..(width * 60) {
            core::ptr::write_volatile(fb_addr.add(i), 0x0000ff);
        }
    }

    if heap_virt_addr == 0 { hcf(); }


    unsafe {
        let test_ptr = heap_virt_addr as *mut u64;
        core::ptr::write_volatile(test_ptr, 0xCAFEBABEDEADBEEF);
        let _test_val = core::ptr::read_volatile(test_ptr);
    }
    // --- HEAP INIT STAGE ---
    init_heap(heap_virt_addr as usize, 1 * 1024 * 1024);

    // BAR 4: GREEN (Heap Initialized without hanging!)
    unsafe {
        for i in (width * 60)..(width * 80) {
            core::ptr::write_volatile(fb_addr.add(i), 0x00ff00);
        }
    }

    // --- FINAL UI SETUP ---
    let font = Font::new(FONT_16X32);
    unsafe {
        // DIRECT DRAWING to avoid backbuffer blit errors for now
        let mut cursor = Cursor::new(fb_addr, fb_addr, fb.width(), fb.height());
        cursor.font = Some(font);
        cursor.clear(0x1a1b26); // Tokyo Night
        UI_CURSOR = Some(cursor);
    }

    println!("Vibe OS: Diagnostic Boot Complete.");
    println!("Dynamic Heap @ {:#x}", heap_virt_addr);

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.color_fg = 0xf7768e;
            println!("\nPANIC: {}", info);
        } else {
            // Emergency Red Screen if UI isn't ready
            if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response().as_ref() {
                if let Some(fb) = fb_response.framebuffers().next() {
                    let fb_addr = fb.addr() as *mut u32;
                    let size = (fb.width() * fb.height()) as usize;
                    for i in 0..size {
                        core::ptr::write_volatile(fb_addr.add(i), 0xf7768e);
                    }
                }
            }
        }
    }
    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
