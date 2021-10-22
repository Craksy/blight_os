use core::{
    alloc::GlobalAlloc,
    mem::{align_of, size_of},
    ptr::null_mut,
};

use super::{align_up, Locked};

struct Node {
    size: usize,
    next: usize,
}

impl Node {
    pub const fn new(size: usize) -> Self {
        Self { size, next: 0 }
    }

    pub fn start(&self) -> usize {
        self as *const Self as usize
    }

    pub fn end(&self) -> usize {
        self.start() + self.size
    }
}

pub struct LinkedListAllocator {
    head: Node,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self { head: Node::new(0) }
    }

    /// Initialize the allocator
    pub fn init(&mut self, start: usize, size: usize) {
        unsafe { self.add_region(start, size) };
    }

    /// create a new free region of `size` bytes at `addr`
    pub unsafe fn add_region(&mut self, addr: usize, size: usize) {
        assert!(size >= size_of::<Node>());
        assert_eq!(align_up(addr, align_of::<Node>()), addr);

        let mut new_node = Node::new(size);
        new_node.next = self.head.next;
        let node_pointer = addr as *mut Node;
        node_pointer.write(new_node);
        self.head.next = addr;
    }

    /// Check if given `region` can hold `requested` space, taking its
    /// `align` into account.
    /// If it can, return the aligned address along with space remaining after
    /// allocation
    fn check_region(region: &mut Node, requested: usize, align: usize) -> Option<(usize, usize)> {
        let start = align_up(region.start(), align);
        let end = start + requested;
        let remainder = region.end().checked_sub(end);
        match remainder {
            Some(r) if r == 0 || r >= size_of::<Node>() => Some((start, r)),
            _ => None,
        }
    }

    /// Traverse the free list, stopping at the first free region that can be
    /// used for an allocation of `size` bytes with given `align`
    /// Return the aligned start address if one is found.
    fn first_fit(&mut self, size: usize, align: usize) -> Option<usize> {
        let mut previous = &mut self.head;
        while previous.next != 0 {
            let current_addr = previous.next;
            let current_node = unsafe { &mut *(current_addr as *mut Node) };

            if let Some((start, remainder)) = Self::check_region(current_node, size, align) {
                previous.next = current_node.next;
                current_node.next = 0;
                if remainder != 0 {
                    unsafe {
                        self.add_region(start + size, remainder);
                    }
                }
                return Some(start);
            }
            previous = current_node;
        }
        None
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut llalloc = self.lock();
        let layout = layout
            .align_to(align_of::<Node>())
            .expect("align failed")
            .pad_to_align();
        let size = layout.size().max(size_of::<Node>());
        let align = layout.align();
        if let Some(start_addr) = llalloc.first_fit(size, align) {
            return start_addr as *mut u8;
        }
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let size = layout
            .align_to(size_of::<Node>())
            .expect("align failed")
            .pad_to_align()
            .size();
        self.lock().add_region(ptr as usize, size);
    }
}
