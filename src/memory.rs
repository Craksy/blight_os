use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

unsafe fn get_active_lvl4_table(physical_offset: VirtAddr) -> &'static mut PageTable {
    let (table_frame, _) = x86_64::registers::control::Cr3::read();
    let physical = table_frame.start_address().as_u64();
    let virtual_add = physical_offset + physical;
    let table_ptr: *mut PageTable = virtual_add.as_mut_ptr();

    &mut *table_ptr
}

pub unsafe fn init(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
    let active_table = get_active_lvl4_table(physical_offset);
    OffsetPageTable::new(active_table, physical_offset)
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let usable_regions = self
            .memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|fr| PhysFrame::containing_address(PhysAddr::new(fr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub fn create_sample_page(
    page: Page,
    mapper: &mut OffsetPageTable,
    allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let map_result = unsafe { mapper.map_to(page, frame, flags, allocator) };
    map_result.expect("Failed to map page").flush();
}
/*
 * Keeping these two around just for reference.
 * They do a pretty good job at illustrating what's actually going on behind the scenes.
pub unsafe fn translate_address(
    virtual_address: VirtAddr,
    physical_offset: VirtAddr,
) -> Option<PhysAddr> {
    translate_address_inner(virtual_address, physical_offset)
}

fn translate_address_inner(
    virtual_address: VirtAddr,
    physical_offset: VirtAddr,
) -> Option<PhysAddr> {
    let (table_frame, _) = x86_64::registers::control::Cr3::read();

    let table_indices = [
        virtual_address.p4_index(),
        virtual_address.p3_index(),
        virtual_address.p2_index(),
        virtual_address.p1_index(),
    ];

    let mut frame = table_frame;

    for &index in &table_indices {
        let virt = physical_offset + frame.start_address().as_u64();
        let table_pointer: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_pointer };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge frame not supported"),
        };
    }
    Some(frame.start_address() + u64::from(virtual_address.page_offset()))
}
*/
