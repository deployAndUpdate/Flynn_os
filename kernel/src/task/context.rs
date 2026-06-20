use crate::memory::stack::MappedStack;

/// Saved CPU context for context switch (x86_64) — voluntary `ret` only.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    pub rsp: u64,
}

pub fn allocate_stack(slot: u64) -> MappedStack {
    MappedStack::allocate(slot)
}

/// Prepare a fresh stack so the first switch ret-jumps into `entry`.
pub fn init_context(stack: &mut [u8], entry: extern "C" fn() -> !) -> TaskContext {
    let stack_bottom = stack.as_mut_ptr() as usize;
    let stack_top = stack_bottom + stack.len();

    let mut sp = stack_top & !0xF;
    sp -= 8;
    unsafe {
        (sp as *mut u64).write(entry as usize as u64);
    }

    TaskContext { rsp: sp as u64 }
}
