use x86_64::structures::paging::PageTableFlags;

use super::layout::{kernel_stack_virt, KERNEL_STACK_SIZE, KERNEL_STACK_SLOTS};
use super::paging;

/// Kernel thread stack backed by mapped physical frames (not heap).
pub struct MappedStack {
    virt_base: usize,
    len: usize,
}

impl MappedStack {
    pub fn allocate(slot: u64) -> Self {
        assert!(
            slot < KERNEL_STACK_SLOTS,
            "kernel stack slot out of range"
        );

        let virt = kernel_stack_virt(slot) as usize;
        paging::with_mapper_and_frames(|mapper, frames| {
            paging::map_region(
                mapper,
                frames,
                x86_64::VirtAddr::new(virt as u64),
                KERNEL_STACK_SIZE,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );
        });

        Self {
            virt_base: virt,
            len: KERNEL_STACK_SIZE,
        }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.virt_base as *mut u8, self.len) }
    }
}
