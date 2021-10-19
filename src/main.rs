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

    print_banner();

    hlt_loop();
}


fn trigger_page_fault(){
    unsafe { *(0xb00b1e5 as *mut u64) = 69 };
}


fn print_banner(){
    println!("{:^80}", "-----------------------------------------------");
    println!("{:^80}", "|                                             |");
    println!("{:^80}", "|          ____  _ _       _     _            |");
    println!("{:^80}", "|         | __ )| (_) __ _| |__ | |_          |");
    println!("{:^80}", "|         |  _ \\| | |/ _` | '_ \\| __|         |");
    println!("{:^80}", "|         | |_) | | | (_| | | | | |_          |");
    println!("{:^80}", "|         |____/|_|_|\\__, |_| |_|\\__|         |");
    println!("{:^80}", "|                    |___/                    |");
    println!("{:^80}", "|                  ___  ____                  |");
    println!("{:^80}", "|                 / _ \\/ ___|                 |");
    println!("{:^80}", "|                | | | \\___ \\                 |");
    println!("{:^80}", "|                | |_| |___) |                |");
    println!("{:^80}", "|                 \\___/|____/                 |");
    println!("{:^80}", "|                                             |");
    println!("{:^80}", "-----------------------------------------------");
    println!("{:^80}", "               __  _                             ");
    println!("{:^80}", "            .-.'  `; `-._  __  _     bah!        ");
    println!("{:^80}", "  bah!     (_,         .-:'  `; `-._/            ");
    println!("{:^80}", "      \\  ,'o\"(        (_,           )            ");
    println!("{:^80}", "        (__,-'      ,'o\"(            )>          ");
    println!("{:^80}", "           (       (__,-'            )           ");
    println!("{:^80}", "            `-'._.--._(             )            ");
    println!("{:^80}", "               |||  |||`-'._.--._.-'             ");
    println!("{:^80}", "                          |||  |||               ");
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


#[test_case]
fn basic_assertion() {
    assert_eq!(1, 1, "1 == 1");
}
