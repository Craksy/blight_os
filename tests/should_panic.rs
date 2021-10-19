#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_runner_entry"]

use core::panic::PanicInfo;

use blight_os::{exit_qemu, serial_print, serial_println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[01;32m[ ✓ ][0m");
    exit_qemu(blight_os::QExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[01;31m[ ✘ ]  Test did not panic![0m");
    exit_qemu(blight_os::QExitCode::Failure);
    loop {}
}

fn should_fail() {
    serial_print!("[01;34m{:.<80}[0m", "should_panic::should_fail");
    assert_eq!(1, 0);
}
