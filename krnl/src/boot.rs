#![no_std]
#![no_main]

extern crate fdt;

use core::panic::PanicInfo;
use core::arch::global_asm;
use core::arch::asm;
use core::fmt::Write;

global_asm!(include_str!("boot.S"));

static mut UART: Option<*mut u8> = None;

#[no_mangle]
fn print(str: &str) {
    unsafe {
    if let Some(u) = UART {
    for &c in str.as_bytes() {
    while (core::ptr::read_volatile(u.add(5)) & 0x20) == 0 {}
    core::ptr::write_volatile(u, c);
    }
}
}
}

#[no_mangle]
pub extern "C" fn kmain(_hartid: usize, fdt_ptr: usize) -> ! {

    let uart = 0x1000_0000 as *mut u8;
    unsafe {
        // Write 'A' directly to UART
        core::ptr::write_volatile(uart, b'A');
    }

    // TEST 2: Check the FDT Pointer alignment
    if fdt_ptr % 8 != 0 {
        // If the pointer is bad, we'd never have survived the fdt crate
        unsafe { core::ptr::write_volatile(uart, b'!'); }
    }

    // Now try the real print
    print(" -> ONISH BOOTED\n\r");

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
    
    print("ONISH");
    unsafe { loop { asm!("wfi") } }
}

const PANIC_UART: *mut u8 = 0x1000_0000 as *mut u8;
struct PanicWriter;
impl Write for PanicWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &b in s.as_bytes() {
            unsafe {
                // Use the HARDCODED address here so we don't depend on FDT
                while (core::ptr::read_volatile(PANIC_UART.add(5)) & 0x20) == 0 {}
                core::ptr::write_volatile(PANIC_UART, b);
            }
        }
        Ok(())
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut writer = PanicWriter;
    let _ = writeln!(writer, "\n\r!!! KERNEL CRASHED !!!");
    // This will now print the EXACT error from the FDT crate
    let _ = writeln!(writer, "{}", info); 
    
    loop {
        unsafe { asm!("wfi") }
    }
}