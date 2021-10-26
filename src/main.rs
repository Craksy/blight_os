#![allow(dead_code)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(blight_os::test_runner)]
#![reexport_test_harness_main = "test_runner_entry"]

extern crate alloc;

use core::panic::PanicInfo;

use alloc::boxed::Box;
use blight_os::{
    hlt_loop,
    memory::BootInfoFrameAllocator,
    println,
    task::{basic_executor::BasicExecutor, Task},
};
use bootloader::{entry_point, BootInfo};
use x86_64::{
    structures::paging::{OffsetPageTable, Page, Translate},
    VirtAddr,
};

entry_point!(kernel_entry);

fn kernel_entry(boot_info: &'static BootInfo) -> ! {
    blight_os::init();
    print_banner();

    let physical_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { blight_os::memory::init(physical_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    blight_os::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap allocation failed.");

    let some_shit_on_the_heap = Box::new(420);
    let mut executor = BasicExecutor::new();
    executor.spawn(Task::new(say_hello()));
    executor.run();

    println!("Thing on the heap: {}", *some_shit_on_the_heap);

    #[cfg(test)]
    test_runner_entry();
    hlt_loop();
}

async fn get_name() -> &'static str {
    &"Bob"
}

async fn say_hello() {
    let name = get_name().await;
    println!("Hello, {}", name);
}

// #[alloc_error_handler]
// fn allocation_error_handler(layout: alloc::alloc::Layout) -> ! {
//     panic!("allocation error: {:?}", layout);
// }

fn translate_some_addresses(mapper: &mut OffsetPageTable, physical_memory_offset: u64) {
    let test_addresses = [0xb8000, 0x201008, 0x010000201a10, physical_memory_offset];

    for add in test_addresses {
        let virt = VirtAddr::new(add);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }
}

fn trigger_page_fault() {
    unsafe { *(0xb00b1e5 as *mut u64) = 69 };
}

#[rustfmt::skip]
fn print_banner(){
    println!("{:^80}", "---------------------------------------------------------");
    println!("{:^80}", "|                                                       |");
    println!("{:^80}", "|               ____  _ _       _     _                 |");
    println!("{:^80}", "|              | __ )| (_) __ _| |__ | |_               |");
    println!("{:^80}", "|              |  _ \\| | |/ _` | '_ \\| __|              |");
    println!("{:^80}", "|              | |_) | | | (_| | | | | |_               |");
    println!("{:^80}", "|              |____/|_|_|\\__, |_| |_|\\__|              |");
    println!("{:^80}", "|                         |___/                         |");
    println!("{:^80}", "|                       ___  ____                       |");
    println!("{:^80}", "|                      / _ \\/ ___|                      |");
    println!("{:^80}", "|                     | | | \\___ \\                      |");
    println!("{:^80}", "|                     | |_| |___) |                     |");
    println!("{:^80}", "|                      \\___/|____/                      |");
    println!("{:^80}", "|                                                       |");
    println!("{:^80}", "---------------------------------------------------------");
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
