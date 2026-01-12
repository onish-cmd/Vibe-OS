#![no_std]
#![no_main]

extern crate fdt;

use core::panic::PanicInfo;
use core::arch::global_asm;
use fdt::Fdt;

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
pub extern "C" fn kmain(_hartid: usize, fdt_ptr: usize) -> ! {
    let fdt = unsafe { fdt::Fdt::from_ptr(fdt_ptr as *const u8) }.unwrap();
    let uart = fdt.chosen()
    .stdout()
    .and_then(|s| s.node().reg())
    .and_then(|mut r| r.next())
    print("NISH");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC");
    loop {}
}