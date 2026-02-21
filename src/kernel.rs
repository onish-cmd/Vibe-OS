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
use limine::request::{FramebufferRequest, MemoryMapRequest, HhdmRequest, StackSizeRequest, RequestsEndMarker, RequestsStartMarker};
use linked_list_allocator::LockedHeap;
use spleen_font::FONT_16X32;
use vibe_framebuffer::{Cursor, Font};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

// --- 1. Request a 64KB Stack ---
// This prevents the font initialization from blowing out the stack.
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

    // Get HHDM Offset safely
    let hhdm_offset = HHDM_REQUEST.get_response()
        .as_ref()
        .expect("VIBE ERROR: HHDM failed")
        .offset();

    // --- 2. Initialize Heap Safely ---
    let response_binding = MEMMAP_REQUEST.get_response();
    
    // 2. Now borrow from that binding
    let memmap_response = response_binding.as_ref().expect("VIBE ERROR: Memmap failed");
    
    let heap_size = 16 * 1024 * 1024; // Lowered to 16MB for stability
    let mut heap_virt_addr: u64 = 0;

    for entry in memmap_response.entries() {
        let raw_type = unsafe { core::mem::transmute::<_, u64>(entry.entry_type) };
        
        // Ensure we are in USABLE memory (0) and NOT in the first 2MB (where kernel/bootloader live)
        if raw_type == 0 && entry.length >= heap_size as u64 && entry.base >= 0x200000 {
            heap_virt_addr = entry.base + hhdm_offset;
            break;
        }
    }

    if heap_virt_addr == 0 { hcf(); }
    init_heap(heap_virt_addr as usize, heap_size);
// --- 3. Initialize Framebuffer ---
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response().as_ref() {
        if let Some(fb) = fb_response.framebuffers().next() {
            let font = Font::new(FONT_16X32);

            // Calculate buffer size: Width * Height * 4 (for u32 pixels)
            let buffer_size = (fb.width() * fb.height() * 4) as usize;
            
            unsafe {
                // ALLOCATE BACKBUFFER FROM HEAP
                let layout = core::alloc::Layout::from_size_align(buffer_size, 4096).unwrap();
                let backbuffer_ptr = alloc::alloc::alloc(layout) as *mut u32;

                if backbuffer_ptr.is_null() {
                    for i in 0..(fb.width * fb.height) {
                        *self.backbuffer_ptr.add(i) = 0xf7768e;
                    }
                    hcf(); // If allocation fails, stop.
                }

                let fb_addr = fb.addr() as *mut u32;
                let mut cursor = Cursor::new(
                    fb_addr,
                    fb_addr, // THE REAL BACKBUFFER
                    fb.width(),
                    fb.height()
                );
                
                cursor.font = Some(font);
                cursor.clear(0x1a1b26);
                UI_CURSOR = Some(cursor);
            }
        }
    }

    println!("Vibe OS: Kernel Initialized.");
    println!("Heap @ {:#x}", heap_virt_addr);

    hcf();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.color_fg = 0xf7768e;
            println!("\nPANIC: {}", info);
        }
    }
    hcf();
}

fn hcf() -> ! {
    loop { unsafe { asm!("hlt") } }
}
