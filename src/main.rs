#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
use core::panic::PanicInfo;
use core::arch::asm;


fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success as u32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0,
    Failed =  u32::MAX,
}

pub fn exit_qemu(exit_code: u32) {
    unsafe {
        asm!(
            "mov eax, {0:e}",
            "out 0xf4, eax",
            in(reg) exit_code,
       );
    }
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    #[cfg(test)]
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}


#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}