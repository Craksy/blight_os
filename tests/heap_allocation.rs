#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(blight_os::test_runner)]
#![reexport_test_harness_main = "test_runner_entry"]

extern crate alloc;

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec::Vec};
use blight_os::{allocator::HEAP_SIZE, memory::BootInfoFrameAllocator};
use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    blight_os::init();

    let physical_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { blight_os::memory::init(physical_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    blight_os::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap allocation failed.");

    test_runner_entry();
    blight_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blight_os::test_panic(info)
}

#[test_case]
fn test_allocation() {
    let value1 = Box::new(42);
    let value2 = Box::new(69);
    assert_eq!(*value1, 42);
    assert_eq!(*value2, 69);
}

#[test_case]
fn large_allocation() {
    let mut v = Vec::new();
    let n = 1000;
    for i in 0..n {
        v.push(i);
    }

    assert_eq!(v.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn reuse_heap_memory() {
    for i in 0..HEAP_SIZE {
        let b = Box::new(i);
        assert_eq!(*b, i);
    }
}
