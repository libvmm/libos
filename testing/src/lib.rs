#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate bootloader;
extern crate rlibc;
extern crate x86_64;
extern crate libos;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[test_case]
fn simple_register_access() {
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[cfg(test)]
entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
