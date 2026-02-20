// Hey! I hope like my project, I'm 11 and coding on a tablet TYSM.
#![no_std]
#![no_main]

extern crate limine;
use limine::request::FramebufferRequest;
use core::arch::asm;
use vibe_framebuffer;

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static mut UI_CURSOR: Option<vibe_framebuffer::Cursor> = None;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_response.framebuffers().next() {
            UI_CURSOR = Some(vibe_framebuffer::Cursor::new(
                fb.addr() as *mut u32, 
                fb.width(), 
                fb.height()
            ));
        }
    }
    clear_screen(0x001A1B26);
    loop { unsafe {
            asm!(
                "hlt"
            )
        }
    }
}

pub fn clear_screen(color: u32) {
    unsafe { 
        if let Some(ref mut cursor) = UI_CURSOR {
            unsafe {
                cursor.clear(color);
            }
        }   
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { 
    loop { 
        unsafe { 
            asm!("hlt")
        } 
    }
}
