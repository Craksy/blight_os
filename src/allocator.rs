extern crate alloc;

pub mod bump;

use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 1024 * 1024;

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

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
