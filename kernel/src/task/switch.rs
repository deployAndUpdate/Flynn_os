use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(
    r#"
    .global switch_task
    switch_task:
        mov [rdi], rsp
        mov rsp, [rsi]
        ret
    "#
);

extern "C" {
    pub fn switch_task(current: *mut TaskContext, next: *const TaskContext);
}
