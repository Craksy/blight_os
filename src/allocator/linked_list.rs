use core::{alloc::GlobalAlloc, mem::size_of};

use super::{HEAP_SIZE, HEAP_START};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

struct LinkedListAllocator {
    head: ListNode,
}

impl ListNode {
    pub const fn new(size: usize) -> Self {
        Self { size, next: None }
    }
}

impl LinkedListAllocator {
    pub fn new() -> Self {
        LinkedListAllocator {
            head: ListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self) {
        self.head.next = Some(&mut *(HEAP_START as *mut ListNode));
        self.head.size = HEAP_SIZE - size_of::<ListNode>();
    }

    pub fn add_region(&self, addr: usize, size: usize) {}
}
