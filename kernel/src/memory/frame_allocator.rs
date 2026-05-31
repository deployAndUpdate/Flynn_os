use bootloader_api::BootInfo;

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

pub fn init_frame_allocator(boot_info: &BootInfo) -> FrameAllocator {
    let (start, end) = usable_regions(&boot_info.memory_regions)
        .next()
        .expect("no usable memory");

    FrameAllocator::new(start, end)
}
