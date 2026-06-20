use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

use super::frame::Frame;
use super::frame_allocator::BootFrameAllocator;

static PHYS_MEM_OFFSET: AtomicU64 = AtomicU64::new(0);

pub fn init_phys_mem_offset(offset: VirtAddr) {
    PHYS_MEM_OFFSET.store(offset.as_u64(), Ordering::SeqCst);
}

pub fn phys_mem_offset() -> VirtAddr {
    VirtAddr::new(PHYS_MEM_OFFSET.load(Ordering::SeqCst))
}

pub unsafe fn active_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::PageTable;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    OffsetPageTable::new(&mut *page_table_ptr, physical_memory_offset)
}

pub fn with_mapper<F>(f: F)
where
    F: FnOnce(&mut OffsetPageTable<'static>),
{
    let offset = phys_mem_offset();
    unsafe {
        let mut mapper = active_page_table(offset);
        f(&mut mapper);
    }
}

pub fn with_mapper_and_frames<F>(f: F)
where
    F: FnOnce(&mut OffsetPageTable<'static>, &mut BootFrameAllocator),
{
    with_mapper(|mapper| {
        let mut frames = BootFrameAllocator;
        f(mapper, &mut frames);
    });
}

pub fn map_heap(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    heap_start: usize,
    heap_size: usize,
) {
    map_region(
        mapper,
        frame_allocator,
        VirtAddr::new(heap_start as u64),
        heap_size,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
}

pub fn map_region(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    virt_start: VirtAddr,
    size: usize,
    flags: PageTableFlags,
) {
    let start_page = Page::containing_address(virt_start);
    let end_addr = virt_start + size as u64 - 1;
    let end_page = Page::containing_address(end_addr);

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .expect("frame allocation failed during mapping");

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("page mapping failed")
                .flush();
        }
    }
}

#[allow(dead_code)]
pub fn map_example(
    page_table: &mut OffsetPageTable,
    page: Page<Size4KiB>,
    frame: Frame,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    use x86_64::{structures::paging::PhysFrame, PhysAddr};

    let phys_frame = PhysFrame::from_start_address(PhysAddr::new(frame.start_address() as u64))
        .expect("frame address must be page-aligned");

    unsafe {
        page_table
            .map_to(page, phys_frame, flags, frame_allocator)
            .expect("map_to failed")
            .flush();
    }
}
