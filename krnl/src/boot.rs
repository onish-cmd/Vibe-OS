#![no_std]
#![no_main]

extern crate fdt;

use core::panic::PanicInfo;
use core::arch::global_asm;
use core::arch::asm;

global_asm!(include_str!("boot.S"));

static mut UART: Option<*mut u8> = None;

#[no_mangle]
fn print(str: &str) {
    unsafe {
    if let Some(u) = UART {
    for &c in str.as_bytes() {
        core::ptr::write_volatile(u, c)
    }
}
}
}

#[no_mangle]
pub extern "C" fn kmain(_hartid: usize, fdt_ptr: usize) -> ! {
    let fdt = unsafe { fdt::Fdt::from_ptr(fdt_ptr as *const u8) }.unwrap();
    let uart: *mut u8 = fdt.chosen()
    .stdout()
    .and_then(|s| s.reg())
    .and_then(|mut r| r.next())
    .map(|reg| reg.starting_address as *mut u8)
    .expect("HARDWARE FAILED");

    unsafe {
        UART = Some(uart)
    }
    
    print("NISH");
    unsafe { loop { asm!("wfi") } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC");
    loop {}
}