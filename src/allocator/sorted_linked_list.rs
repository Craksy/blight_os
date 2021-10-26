use core::{
    alloc::GlobalAlloc,
    mem::{align_of, size_of},
    ptr::{null_mut, NonNull},
};

use super::{align_up, Locked};

/// A node of the freelist.
pub struct Node {
    size: usize,
    next: Option<NonNull<Node>>,
}

unsafe impl Send for Node {}

impl core::fmt::Debug for Node {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("")
            .field(if &self.size == &0 {
                &"HEAD"
            } else {
                &self.size
            })
            .field(unsafe {
                if let Some(node) = &self.next {
                    node.as_ref()
                } else {
                    &"Nil"
                }
            })
            .finish()
    }
}

impl Node {
    pub const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    pub fn start(&self) -> usize {
        self as *const Self as usize
    }

    pub fn end(&self) -> usize {
        self.start() + self.size
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
    unsafe fn add_region_sorted(&mut self, mut addr: NonNull<Node>, size: usize) {
        assert!(size >= size_of::<Node>());
        assert_eq!(
            align_up(addr.as_ptr() as usize, align_of::<Node>()),
            addr.as_ptr() as usize
        );

        let mut previous = &mut self.head;
        // let mut current = &mut *(previous.next as *mut Node);
        let mut current = previous.next.unwrap().as_mut();
        while current.next.is_some() && current.next.unwrap() > addr {
            previous = current;
            current = previous.next.unwrap().as_mut()
        }

        // If the previous node neighbours the added region, simply expand it
        // and consider that the "new node"
        let mut start_node = if previous.end() == addr.as_ptr() as usize && previous.size > 0 {
            previous.size += size;
            &mut previous
        } else {
            let new_node = addr.as_mut();
            *new_node = Node::new(size);
            previous.next = Some(addr);
            addr.as_mut()
        };

        if current.start() == start_node.end() {
            start_node.size += current.size;
            start_node.next = current.next;
        } else {
            start_node.next = NonNull::new(current);
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

    fn split_region(region: &mut Node, size: usize) -> Option<NonNull<Node>> {
        let start = region.end() - size;
        let new_node_ptr = NonNull::new(start as *mut Node);
        let mut new_node = unsafe { new_node_ptr.unwrap().as_mut() };
        *new_node = Node::new(size);
        new_node.next = region.next;
        new_node_ptr
    }

    /// Traverse the free list, stopping at the first free region that can be
    /// used for an allocation of `size` bytes with given `align`
    /// Return the aligned start address if one is found.
    fn first_fit(&mut self, size: usize, align: usize) -> Option<usize> {
        let mut previous = &mut self.head;
        while let Some(mut next) = previous.next {
            let current_node = unsafe { next.as_mut() };
            if let Some((start, remainder)) = Self::check_region(current_node, size, align) {
                previous.next = if remainder != 0 {
                    Self::split_region(current_node, remainder)
                } else {
                    current_node.next
                };
                return Some(start);
            }
            previous = current_node;
        }
        None
    }

    pub fn get_length(&mut self) -> usize {
        let mut current = &mut self.head;
        let mut count = 1;
        while let Some(mut next) = current.next {
            count += 1;
            current = unsafe { next.as_mut() };
        }
        count
    }

    pub fn get_free_space(&mut self) -> usize {
        let mut current = &mut self.head;
        let mut space = current.size;
        while let Some(mut next) = current.next {
            current = unsafe { next.as_mut() };
            space += current.size;
        }
        space
    }

    pub fn init(&mut self, start: usize, size: usize) {
        let node_ptr = NonNull::new(start as *mut Node);
        self.head.next = node_ptr;
        let new_node = unsafe { node_ptr.expect("Null pointer").as_mut() };
        *new_node = Node::new(size);
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
        self.lock()
            .add_region_sorted(NonNull::new(ptr as *mut Node).expect("Null ptr"), size);
    }
}
