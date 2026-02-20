// Hey! I hope like my project, I'm 11 and coding on a tablet TYSM.
#![no_std]
#![no_main]

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
use limine::request::FramebufferRequest;
use core::arch::asm;
use vibe_framebuffer::Cursor;
use spleen_font::FONT_16X32;
use core::fmt::{self, Write};
use limine::request::MemoryMapRequest;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(start: usize, size: usize) {
    unsafe {
        ALLOCATOR.lock().init(start, size);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("VIBE OS: Heap Allocation Error - Layout: {:?}", layout);
}

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static mut UI_CURSOR: Option<Cursor> = None;

pub fn _print(args: fmt::Arguments) {
    unsafe {
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.write_fmt(args).unwrap();
        }
    }
}
pub fn clear_screen(color: u32) {
    unsafe { 
        if let Some(ref mut cursor) = UI_CURSOR {
            cursor.clear(color);
        }   
    }
}

static MEMMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let memmap = MEMMAP_REQUEST.get_response().get().memmap();
    let heap_size = 32 * 1024 * 1024;
    let mut heap_addr: u64 = 0;

    for entry in memmap {
        if entry.typ == limine::MemoryMapEntryType::USABLE && entry.len >= heap_size as u64 {
            heap_addr = entry.base;
            break; 
        }
    }
    if heap_addr == 0 { panic!("Could not find enough RAM for Vibe OS heap!"); }
    crate::allocator::init_heap(heap_addr as usize, heap_size);
    unsafe { 
        if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
            if let Some(fb) = fb_response.framebuffers().next() {
                let font = vibe_framebuffer::Font::new(FONT_16X32);
                let mut cursor = Cursor::new(
                    fb.addr() as *mut u32,
                    fb.width(),
                    fb.height()
                );
                
                cursor.font = Some(font); // Attach font to the local variable
                cursor.clear(cursor.color);
                UI_CURSOR = Some(cursor); // Now move it to the global static
            }
        }
    }

    extern crate alloc;
    use alloc::string::String;
    use alloc::vec::Vec;

    let mut vibe_list = Vec::new();
    vibe_list.push(String::from("Vibe"));
    vibe_list.push(String::from("OS"));

    println!("Heap Initialized! Data: {} {}", vibe_list[0], vibe_list[1]);

    loop { unsafe { asm!("hlt") } }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop { unsafe { asm!("hlt") } }
}
