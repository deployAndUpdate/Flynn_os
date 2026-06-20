//! Fixed virtual addresses for kernel subsystems.

/// Kernel heap (linked_list_allocator).
pub const KERNEL_HEAP_START: usize = 0x4444_4444_0000;
pub const KERNEL_HEAP_SIZE: usize = 1024 * 1024;

/// Mapped kernel thread stacks (high canonical half).
pub const KERNEL_STACK_BASE: u64 = 0xFFFF_A000_0000_0000;
pub const KERNEL_STACK_SIZE: usize = 4096 * 8;
/// Max concurrent mapped stacks (bitmap slot index passed to allocate_stack).
pub const KERNEL_STACK_SLOTS: u64 = 64;

pub fn kernel_stack_virt(slot: u64) -> u64 {
    KERNEL_STACK_BASE + slot * KERNEL_STACK_SIZE as u64
}
