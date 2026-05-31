use bootloader_api::BootInfo;
use x86_64::structures::paging::{FrameAllocator as X86FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

use super::frame::Frame;
use super::memory_map::usable_regions;

pub struct FrameAllocator {
    next: usize,
    end: usize,
}

impl FrameAllocator {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            next: start / super::frame::PAGE_SIZE,
            end: end / super::frame::PAGE_SIZE,
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        if self.next >= self.end {
            return None;
        }

        let frame = Frame {
            number: self.next,
        };
        self.next += 1;
        Some(frame)
    }
}

pub struct BootFrameAllocator(pub FrameAllocator);

unsafe impl X86FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.0.allocate_frame().map(|frame| {
            PhysFrame::from_start_address(PhysAddr::new(frame.start_address() as u64))
                .expect("allocated frame must be page-aligned")
        })
    }
}

pub fn init_frame_allocator(boot_info: &BootInfo) -> BootFrameAllocator {
    let (start, end) = usable_regions(&boot_info.memory_regions)
        .max_by_key(|(start, end)| end.saturating_sub(*start))
        .expect("no usable memory");

    BootFrameAllocator(FrameAllocator::new(start, end))
}
