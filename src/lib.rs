#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use core::arch::asm;
use core::ops::Fn;

pub mod serial;
pub mod vga_buffer;

#[repr(u32)]
pub enum QemuExitCode {
    Success = 0,
    Failed =  u32::MAX,
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn exit_qemu(exit_code: u32) {
    unsafe { asm!("out 0xf4, eax",in("eax") exit_code); }
}


pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success as u32);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\nError: {}", info);
    exit_qemu(QemuExitCode::Failed as u32);
    loop {}
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}