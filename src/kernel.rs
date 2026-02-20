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

#[no_mangle]
pub extern "C" fn _start() -> ! {
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
                clear_screen(cursor.color);
                UI_CURSOR = Some(cursor); // Now move it to the global static
            }
        }
    }
    println!("Vibe OS is alive!");
    let numx = 2;
    let numy = 4;
    let result = numx + numy;
    println!("fmt test: {} + {} = {}", numx, numy, result);
    println!("Lets panic!");
    panic!();
    
    loop { unsafe { asm!("hlt") } }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop { unsafe { asm!("hlt") } }
}
