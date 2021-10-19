#![allow(dead_code)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blight_os::test_runner)]
#![reexport_test_harness_main = "test_runner_entry"]

use blight_os::hlt_loop;
#[allow(unused_imports)]
use blight_os::{exit_qemu, println, QExitCode};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    blight_os::init();

    #[cfg(test)]
    test_runner_entry();

    println!("Hello, World!");
    x86_64::instructions::interrupts::int3();
    println!("after interrupt");
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blight_os::test_panic(info)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("WOuPeR dOopEr. Looks like someone made a wittle little fucky wucky.:");
    println!("{}", info);
    hlt_loop();
}

// tests

#[test_case]
fn basic_assertion() {
    assert_eq!(1, 1, "1 == 1");
}

// end of tests
