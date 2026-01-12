#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::global_asm;

global_asm!(include_str!("boot.S"));

#[no_mangle]
fn print(str: &str) {
    let uart = 0x1000_0000 as *mut u8;
    unsafe {
    for &c in str.as_bytes() {
        core::ptr::write_volatile(uart, c)
    }
}
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    print("NISH");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC");
    loop {}
}