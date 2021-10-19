#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(type_ascription)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_runner_entry"]

extern crate alloc;
extern crate num_derive;

use bootloader::{entry_point, BootInfo};
use core::{any::type_name, panic::PanicInfo};

pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

#[repr(u32)]
pub enum QExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        // the IO port-ID `0xF4` represents QEMU's internal escape hatch, or "exit device"
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Central place for initialisation
pub fn init() {
    gdt::init();
    interrupts::init_descriptor_table();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic(info)
}

#[cfg(test)]
entry_point!(kernel_test_entry);
/// Entry point for `cargo test --lib`
#[cfg(test)]
pub fn kernel_test_entry(_boot_info: &'static BootInfo) -> ! {
    init();
    test_runner_entry();
    hlt_loop();
}

/// Trait for test functions that prints their name, invokes them, and prints a
/// success status message if it didn't panic
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        let name: &str = type_name::<T>().split("::").last().unwrap();
        serial_print!("[01;34m{:.<80}[0m", name);
        self();
        serial_println!("[01;32m[ ✓ ][0m");
    }
}

/// Panic handler for the custom testing framework
pub fn test_panic(_info: &PanicInfo) -> ! {
    serial_println!("[01;31m[ ✘ ][0m");
    serial_println!("   [01;31m┌{:─<78}┐[0m", "");
    serial_println!("   [01;31m│{:^78}│[0m", "x Test failed x");
    serial_println!("   [01;31m└{:─<78}┘[0m", "");
    serial_println!("{:^85}", _info);
    exit_qemu(QExitCode::Failure);
    hlt_loop();
}

/// Test runner for the custom testing framework
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    serial_println!("   [01;32m┌{:─<78}┐[0m", "");
    serial_println!("   [01;32m│{:^78}│[0m", "✓ All tests passed! ✓");
    serial_println!("   [01;32m└{:─<78}┘[0m", "");
    exit_qemu(QExitCode::Success);
}

#[test_case]
fn test_breakpoint_exception_handler() {
    x86_64::instructions::interrupts::int3();
}
