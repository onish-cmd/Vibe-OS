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

extern crate limine;
extern crate alloc;

use core::arch::asm;
use core::fmt::{self, Write};
use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, HhdmRequest, RequestsEndMarker, RequestsStartMarker};
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

pub fn init_heap(start: usize, size: usize) {
    unsafe { ALLOCATOR.lock().init(start as *mut u8, size); }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("VIBE OS: Heap Allocation Error - Layout: {:?}", layout);
}

static mut UI_CURSOR: Option<Cursor> = None;

pub fn _print(args: fmt::Arguments) {
    unsafe { if let Some(ref mut cursor) = UI_CURSOR { let _ = cursor.write_fmt(args); } }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    // --- 1. Get HHDM Offset ---
    // This is crucial for Zen 4 / Modern UEFI systems to map physical memory to virtual space
    let hhdm_offset = HHDM_REQUEST.get_response()
        .as_ref()
        .expect("VIBE ERROR: HHDM response failed")
        .offset();

    // --- 2. Initialize Heap ---
    let memmap_binding = MEMMAP_REQUEST.get_response();
    let memmap_response = memmap_binding.as_ref().expect("VIBE ERROR: Memmap response failed");
    
    let heap_size = 32 * 1024 * 1024; // 32MB
    let mut heap_virt_addr: u64 = 0;

    for entry in memmap_response.entries() {
        let raw_type = unsafe { core::mem::transmute::<_, u64>(entry.entry_type) };
        
        // Type 0 is USABLE. We skip the first 1MB to be safe.
        if raw_type == 0 && entry.length >= heap_size as u64 && entry.base >= 0x100000 {
            heap_virt_addr = entry.base + hhdm_offset;
            break;
        }
    }

    if heap_virt_addr == 0 { hcf(); }
    init_heap(heap_virt_addr as usize, heap_size);

    // --- 3. Initialize Graphics ---
    let fb_binding = FRAMEBUFFER_REQUEST.get_response();
    if let Some(fb_response) = fb_binding.as_ref() {
        if let Some(fb) = fb_response.framebuffers().next() {
            let font = Font::new(FONT_16X32);
            
            // Map the framebuffer address through HHDM as well
            // (Though Limine usually provides a virtual address for fb.addr())
            unsafe {
                let mut cursor = Cursor::new(
                    fb.addr() as *mut u32, 
                    core::ptr::null_mut(), 
                    fb.width(), 
                    fb.height()
                );
                cursor.font = Some(font);
                cursor.clear(0x1a1b26); // Tokyo Night Background
                UI_CURSOR = Some(cursor);
            }
        }
    }

    println!("Vibe OS: Kernel Space Initialized.");
    println!("HHDM Offset: {:#x}", hhdm_offset);
    println!("Heap: 32MB @ Virtual {:#x}", heap_virt_addr);

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.color_fg = 0xf7768e; // Tokyo Night Red
            cursor.x = 0;
            println!("\n[ VIBE OS FATAL ERROR ]\n{}", info);
        }
    }
    hcf();
}

fn hcf() -> ! {
    loop { unsafe { asm!("hlt") } }
}
