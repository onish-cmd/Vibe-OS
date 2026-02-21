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

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    // 1. Initial FB Setup
    let fb_binding = FRAMEBUFFER_REQUEST.get_response();
    let fb_res = fb_binding.as_ref().expect("FB Failed");
    let fb = fb_res.framebuffers().next().expect("No FB");
    let fb_addr = fb.addr() as *mut u32;
    let width = fb.width() as usize;

    // BAR 1: WHITE
    unsafe { for i in 0..(width * 20) { core::ptr::write_volatile(fb_addr.add(i), 0xffffff); } }

    // 2. HHDM Setup
    let hhdm_binding = HHDM_REQUEST.get_response();
    let hhdm_offset = hhdm_binding.as_ref().expect("HHDM Failed").offset();

    // BAR 2: YELLOW
    unsafe { for i in (width * 20)..(width * 40) { core::ptr::write_volatile(fb_addr.add(i), 0xffff00); } }

    // 3. Memmap & Visual Debugger
    let memmap_binding = MEMMAP_REQUEST.get_response();
    let memmap_response = memmap_binding.as_ref().expect("Memmap Failed");

    let heap_size = 16 * 1024 * 1024;
    let mut heap_virt_addr: u64 = 0;

    // Iterate through entries to draw debug squares and find heap
    for (i, entry) in memmap_response.entries().iter().enumerate() {
        // Draw squares: Green = Usable, Red = Reserved
        let color = if entry.entry_type == EntryType::USABLE { 0x9ece6a } else { 0xf7768e };
        unsafe {
            for x in (i * 30)..(i * 30 + 20) {
                for y in 100..120 {
                    core::ptr::write_volatile(fb_addr.add(y * width + x), color);
                }
            }
        }

        // Logic to pick the heap
        if entry.entry_type == EntryType::USABLE && entry.base >= 0x1000000 && heap_virt_addr == 0 {
            if entry.length >= heap_size as u64 {
                heap_virt_addr = entry.base + hhdm_offset;
            }
        }
    }

    // BAR 3: BLUE
    unsafe { for i in (width * 40)..(width * 60) { core::ptr::write_volatile(fb_addr.add(i), 0x0000ff); } }

    if heap_virt_addr == 0 { hcf(); }

    // TEST WRITE: If it hangs here, the page is not mapped!
    unsafe { core::ptr::write_volatile(heap_virt_addr as *mut u8, 0xAA); }

    init_heap(heap_virt_addr as usize, heap_size);

    // BAR 4: GREEN
    unsafe { for i in (width * 60)..(width * 80) { core::ptr::write_volatile(fb_addr.add(i), 0x00ff00); } }

    // Final UI
    let font = Font::new(FONT_16X32);
    unsafe {
        let mut cursor = Cursor::new(fb_addr, fb_addr, fb.width(), fb.height());
        cursor.font = Some(font);
        cursor.clear(0x1a1b26);
        UI_CURSOR = Some(cursor);
    }

    println!("Vibe OS: Dynamic Heap Active!");
    println!("Heap Base: {:#x}", heap_virt_addr);

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.color_fg = 0xf7768e;
            println!("\nPANIC: {}", info);
        } else {
            // Emergency Red
            if let Some(fb_res) = FRAMEBUFFER_REQUEST.get_response().as_ref() {
                if let Some(fb) = fb_res.framebuffers().next() {
                    let addr = fb.addr() as *mut u32;
                    for i in 0..(fb.width() * fb.height()) as usize {
                        core::ptr::write_volatile(addr.add(i), 0xf7768e);
                    }
                }
            }
        }
    }
    hcf();
}

fn hcf() -> ! {
    loop { unsafe { asm!("hlt") } }
}
