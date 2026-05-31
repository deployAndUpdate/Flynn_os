use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

use super::frame::Frame;

pub unsafe fn active_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    OffsetPageTable::new(&mut *page_table_ptr, physical_memory_offset)
}

pub fn map_heap(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    heap_start: usize,
    heap_size: usize,
) {
    let heap_start_addr = VirtAddr::new(heap_start as u64);
    let heap_end_addr = heap_start_addr + heap_size as u64 - 1;

    let start_page = Page::containing_address(heap_start_addr);
    let end_page = Page::containing_address(heap_end_addr);

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .expect("frame allocation failed during heap mapping");

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("heap page mapping failed")
                .flush();
        }
    }
}

pub fn map_example(
    page_table: &mut OffsetPageTable,
    page: Page<Size4KiB>,
    frame: Frame,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let phys_frame = PhysFrame::from_start_address(PhysAddr::new(frame.start_address() as u64))
        .expect("frame address must be page-aligned");

    unsafe {
        page_table
            .map_to(page, phys_frame, flags, frame_allocator)
            .expect("map_to failed")
            .flush();
    }
}
