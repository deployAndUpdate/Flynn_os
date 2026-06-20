use core::mem::{align_of, size_of};

use x86_64::registers::rflags::RFlags;
use x86_64::structures::idt::InterruptStackFrame;

use crate::memory::stack::MappedStack;

/// Saved CPU context for context switch (x86_64).
///
/// Voluntary switch: `rsp` → return address slot, `preempted = 0` → `ret`.
/// Preempted switch: `rsp` → `InterruptStackFrame`, `preempted = 1` → `iretq`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    pub rsp: u64,
    pub preempted: u8,
    _pad: [u8; 7],
}

impl TaskContext {
    pub const VOLUNTARY: u8 = 0;
    pub const PREEMPTED: u8 = 1;

    pub fn voluntary(rsp: u64) -> Self {
        Self {
            rsp,
            preempted: Self::VOLUNTARY,
            _pad: [0; 7],
        }
    }
}

pub fn allocate_stack(slot: u64) -> MappedStack {
    MappedStack::allocate(slot)
}

/// Prepare a fresh stack so the first switch `ret`-jumps into `entry`.
pub fn init_context(stack: &mut [u8], entry: extern "C" fn() -> !) -> TaskContext {
    let stack_bottom = stack.as_mut_ptr() as usize;
    let stack_top = stack_bottom + stack.len();

    let mut sp = stack_top & !0xF;
    sp -= 8;
    unsafe {
        (sp as *mut u64).write(entry as usize as u64);
    }

    TaskContext::voluntary(sp as u64)
}

/// Copy interrupt frame to a fixed slot at the stack bottom for later `iretq`.
///
/// The CPU clears IF on interrupt entry; the saved frame must re-enable interrupts
/// on `iretq` or timer/keyboard IRQs stop for that task.
pub fn save_preempt_frame(stack: &MappedStack, frame: &InterruptStackFrame) -> TaskContext {
    let frame_size = size_of::<InterruptStackFrame>();
    let align = align_of::<InterruptStackFrame>();
    let mut sp = stack.bottom();
    if sp % align != 0 {
        sp += align - (sp % align);
    }

    unsafe {
        core::ptr::copy_nonoverlapping(
            frame as *const InterruptStackFrame as *const u8,
            sp as *mut u8,
            frame_size,
        );
        // RIP (8) + CS (8) → RFLAGS at offset 16 in InterruptStackFrame.
        const RFLAGS_OFFSET: usize = 16;
        let flags = (sp as *mut u8).add(RFLAGS_OFFSET) as *mut u64;
        *flags |= RFlags::INTERRUPT_FLAG.bits();
    }

    TaskContext {
        rsp: sp as u64,
        preempted: TaskContext::PREEMPTED,
        _pad: [0; 7],
    }
}
