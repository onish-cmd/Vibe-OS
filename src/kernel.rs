#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

extern crate alloc;
extern crate limine;

use core::arch::asm;
use core::fmt::{self, Write};
use limine::request::{
    FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker,
};
use limine::BaseRevision;
use linked_list_allocator::LockedHeap;
use spleen_font::FONT_16X32;
use vibe_framebuffer::{Cursor, Font};

// --- Limine Protocol Tags ---
#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
// ----------------------------

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(start: usize, size: usize) {
    unsafe {
        ALLOCATOR.lock().init(start as *mut u8, size);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("VIBE OS: Heap Allocation Error - Layout: {:?}", layout);
}

static mut UI_CURSOR: Option<Cursor> = None;

pub fn _print(args: fmt::Arguments) {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            let _ = cursor.write_fmt(args);
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Ensure Limine is happy
    assert!(BASE_REVISION.is_supported());

    // Fix: Accessing MemoryMapEntryType directly from the response module
    let memmap_response = unsafe {
        MEMMAP_REQUEST
            .get_response()
            .as_ref()
            .expect("Memmap request failed")
    };
    let entries = memmap_response.entries();

    let entries = unsafe {
        memmap_response
            .get_memory_map()
            .expect("Failed to get memory map entries")
    };

    let heap_size = 32 * 1024 * 1024;
    let mut heap_addr: u64 = 0;

    for entry in entries {
        if entry.typ == limine::structures::memory_map_entry::MemoryMapEntryType::Usable
            && entry.length >= heap_size as u64
        {
            heap_addr = entry.base;
            break;
        }
    }

    if heap_addr == 0 {
        panic!("Could not find enough RAM for Vibe OS heap!");
    }

    init_heap(heap_addr as usize, heap_size);

    unsafe {
        if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
            if let Some(fb) = fb_response.framebuffers().next() {
                let font = Font::new(FONT_16X32);

                let mut cursor = Cursor::new(
                    fb.addr() as *mut u32,
                    core::ptr::null_mut(),
                    fb.width(),
                    fb.height(),
                );

                cursor.font = Some(font);
                cursor.clear(0x1a1b26); // Tokyo Night "Night"
                UI_CURSOR = Some(cursor);
            }
        }
    }

    use alloc::string::String;
    use alloc::vec::Vec;

    let mut vibe_list = Vec::new();
    vibe_list.push(String::from("Vibe"));
    vibe_list.push(String::from("OS"));

    println!("Heap Initialized! Data: {} {}", vibe_list[0], vibe_list[1]);

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.color_fg = 0xf7768e;
            cursor.x = 0;
            println!("\n[ VIBE OS FATAL ERROR ]");
            println!("------------------------");
            println!("{}", info);
        }
    }
    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
        }
    }
}
