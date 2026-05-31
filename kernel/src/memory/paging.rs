use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
    Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

use super::frame::Frame;

pub struct FrameAllocatorStub;

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorStub {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub unsafe fn active_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    OffsetPageTable::new(&mut *page_table_ptr, physical_memory_offset)
}

pub fn map_example(page_table: &mut OffsetPageTable, page: Page, frame: Frame) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let phys_frame = PhysFrame::from_start_address(PhysAddr::new(frame.start_address() as u64))
        .expect("frame address must be page-aligned");

    unsafe {
        page_table
            .map_to(page, phys_frame, flags, &mut FrameAllocatorStub)
            .expect("map_to failed")
            .flush();
    }
}
