use core::{
    alloc::GlobalAlloc,
    mem::{align_of, size_of},
    ptr::null_mut,
};

use super::{align_up, Locked, HEAP_START};

pub struct Node {
    size: usize,
    next: usize,
}

impl core::fmt::Debug for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("")
            .field(if &self.size == &0 {
                &"HEAD"
            } else {
                &self.size
            })
            .field(&format_args!(
                "{:#4x}",
                &self.next.saturating_sub(HEAP_START)
            ))
            .field(unsafe {
                if let Some(node) = &self.next_node() {
                    node
                } else {
                    &"Nil"
                }
            })
            .finish()
    }
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

    pub unsafe fn next_node(&self) -> Option<&mut Node> {
        if self.next == 0 {
            None
        } else {
            Some(&mut *(self.next as *mut Node))
        }
    }
}

pub struct SortedLinkedListAllocator {
    head: Node,
}

impl SortedLinkedListAllocator {
    pub const fn new() -> Self {
        Self { head: Node::new(0) }
    }

    /// create a new free region of `size` bytes at `addr`
    unsafe fn add_region_sorted(&mut self, addr: usize, size: usize) {
        assert!(size >= size_of::<Node>());
        assert_eq!(align_up(addr, align_of::<Node>()), addr);

        let mut previous = &mut self.head;
        let mut current = &mut *(previous.next as *mut Node);
        while current.next != 0 && current.next > addr {
            previous = current;
            current = &mut *(previous.next as *mut Node)
        }

        // If the previous node neighbours the added region, simply expand it
        // and consider that the "new node"
        let mut start_node = if previous.end() == addr && previous.size > 0 {
            previous.size += size;
            &mut previous
        } else {
            let new_node: Node;
            new_node = Node::new(size);
            let node_ptr = addr as *mut Node;
            previous.next = addr;
            node_ptr.write(new_node);
            &mut *node_ptr
        };

        if current.start() == start_node.end() {
            start_node.size += current.size;
            start_node.next = current.next;
        } else {
            start_node.next = current.start();
        }
    }

    /// Check if given `region` can hold `requested` space, taking its
    /// `align` into account.
    /// If it can, return the aligned address along with space remaining after
    fn check_region(region: &mut Node, requested: usize, align: usize) -> Option<(usize, usize)> {
        let start = align_up(region.start(), align);
        let end = start + requested;
        let remainder = region.end().checked_sub(end);
        match remainder {
            Some(r) if r == 0 || r >= size_of::<Node>() => Some((start, r)),
            _ => None,
        }
    }

    fn split_region<'a>(previous: &'a mut Node, region: &'a mut Node, size: usize) -> &'a mut Node {
        let mut new_node = Node::new(size);
        let start = region.end() - size;
        new_node.next = region.next;
        previous.next = start;
        let ptr = start as *mut Node;
        unsafe { ptr.write(new_node) }
        unsafe { &mut *ptr }
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
                if remainder != 0 {
                    previous.next = start;
                    Self::split_region(previous, current_node, remainder);
                } else {
                    previous.next = current_node.next;
                }
                return Some(start);
            }
            previous = current_node;
        }
        None
    }

    pub fn get_length(&mut self) -> usize {
        let mut current = &mut self.head;
        let mut count = 1;
        while let Some(next) = unsafe { current.next_node() } {
            count += 1;
            current = next;
        }
        count
    }

    pub fn get_free_space(&mut self) -> usize {
        let mut current = &mut self.head;
        let mut space = current.size;
        while let Some(next) = unsafe { current.next_node() } {
            current = next;
            space += current.size;
        }
        space
    }

    pub fn init(&mut self, start: usize, size: usize) {
        let new_node = Node::new(size);
        let node_ptr = start as *mut Node;
        self.head.next = start;
        unsafe { node_ptr.write(new_node) };
    }

    pub fn debug_print(&self) {
        crate::println!("{:?}", self.head);
    }
}

unsafe impl GlobalAlloc for Locked<SortedLinkedListAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut llalloc = self.lock();
        let layout = layout
            .align_to(align_of::<Node>())
            .expect("align failed")
            .pad_to_align();
        let size = layout.size().max(size_of::<Node>());
        let align = layout.align();
        match llalloc.first_fit(size, align) {
            Some(start_addr) => start_addr as *mut u8,
            _ => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let size = layout
            .align_to(size_of::<Node>())
            .expect("align failed")
            .pad_to_align()
            .size();
        self.lock().add_region_sorted(ptr as usize, size);
    }
}
