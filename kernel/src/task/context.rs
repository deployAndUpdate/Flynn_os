use x86_64::structures::idt::InterruptStackFrame;

use crate::memory::stack::MappedStack;

/// Saved CPU context for voluntary context switch (x86_64).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    pub rsp: u64,
    pub preempted: u8,
    _pad: [u8; 7],
}

impl TaskContext {
    pub const VOLUNTARY: u8 = 0;

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

/// Prepare a fresh stack so the first switch jumps into `entry`.
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

// Reserved for future true-ISR preempt (Phase 2.1b).
#[allow(dead_code)]
pub fn save_preempt_frame(_frame: &InterruptStackFrame) -> TaskContext {
    TaskContext::voluntary(0)
}
