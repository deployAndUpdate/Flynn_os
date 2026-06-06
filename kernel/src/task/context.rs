/// Saved CPU context for cooperative context switch (x86_64).
///
/// Only RSP is saved — `switch_context` swaps stacks and returns into
/// the target task via the address pushed at stack init.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    pub rsp: u64,
}

const MIN_STACK_SIZE: usize = 4096 * 4;

pub fn allocate_stack() -> alloc::vec::Vec<u8> {
    let mut stack = alloc::vec::Vec::with_capacity(MIN_STACK_SIZE);
    stack.resize(MIN_STACK_SIZE, 0);
    stack
}

/// Prepare a fresh stack so the first `switch_context` ret-jumps into `entry`.
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
