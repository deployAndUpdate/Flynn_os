use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(
    r#"
    .global switch_context
    switch_context:
        mov [rdi], rsp
        mov rsp, [rsi]
        ret
    "#
);

extern "C" {
    pub fn switch_context(current: *mut TaskContext, next: *const TaskContext);
}
