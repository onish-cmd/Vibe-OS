// Hey! I hope like my project, I'm 11 and coding on a tablet TYSM.
#![no_std]
#![no_main]

extern crate limine;
use limine::request::FramebufferRequest;
use core::arch::asm;
use framebuffer;

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(fb) = fb_response.framebuffers().next() {
            let mut cursor = framebuffer::Cursor::new(
                fb.addr() as *mut u32, 
                fb.width(), 
                fb.height()
            );
        }
    }
    framebuffer::clear(0x1a1b26)
    loop { unsafe {
            asm!(
                "hlt"
            )
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
