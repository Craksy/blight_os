extern crate alloc;

pub mod bump;
pub mod linked_list;
pub mod sorted_linked_list;

use spin::{Mutex, MutexGuard};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use self::sorted_linked_list::SortedLinkedListAllocator;

#[global_allocator]
pub static ALLOCATOR: Locked<SortedLinkedListAllocator> =
    Locked::new(SortedLinkedListAllocator::new());
// pub static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

//use self::bump::BumpAllocator;
//pub static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 50 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let start_add = VirtAddr::new(HEAP_START as u64);
        let end_add = start_add + HEAP_SIZE - 1 as u64;
        let start_page = Page::containing_address(start_add);
        let end_page = Page::containing_address(end_add);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);

    Ok(())
}

pub fn align_up(addr: usize, align: usize) -> usize {
    // super smart piece of bit magic which is actually a lot faster:
    // (addr + align -1) & !(align-1)
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
}

// Thin wrapper around Mutex.
// Its purpose is to get around the rule of implementing traits for external crates.
// By making a crate-local wrapper traits can be implemented for it, making it
// possible to get mutable references for variables that are otherwise immutable
// only
pub struct Locked<T> {
    inner: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock()
    }
}
