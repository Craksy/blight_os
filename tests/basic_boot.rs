#![allow(dead_code)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blight_os::test_runner)]
#![reexport_test_harness_main = "test_runner_entry"]

use core::panic::PanicInfo;

use blight_os::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blight_os::test_panic(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_runner_entry();
    loop {}
}

#[test_case]
fn test_println() {
    println!("Testing that println works upon boot")
}
