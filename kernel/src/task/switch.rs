use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(
    r#"
    .global switch_task
    .global switch_to
    .global first_task_run

    // void switch_task(TaskContext* current, TaskContext* next)
    // Voluntary switch from task context — RFLAGS (incl. IF) stay as-is; plain ret.
    switch_task:
        mov [rdi], rsp
        mov byte ptr [rdi + 8], 0
        mov rax, [rsi]
        mov rsp, rax
        cmp byte ptr [rsi + 8], 0
        je 1f
        iretq
    1:  ret

    // void switch_to(TaskContext* next) — called from timer ISR; must iretq into task.
    switch_to:
        mov rax, [rdi]
        mov rsp, rax
        cmp byte ptr [rdi + 8], 0
        je 2f
        iretq
    2:  // Voluntary slot: [rsp] = return address → build synthetic interrupt frame.
        pop rcx
        mov rdx, rsp
        xor eax, eax
        mov ax, ss
        push rax
        push rdx
        pushfq
        pop rax
        or rax, 0x200
        push rax
        xor eax, eax
        mov ax, cs
        push rax
        push rcx
        iretq

    // void first_task_run(TaskContext* next) — bootstrap from kernel with IF=1.
    first_task_run:
        mov rax, [rdi]
        mov rsp, rax
        cmp byte ptr [rdi + 8], 0
        je 3f
        iretq
    3:  ret
    "#
);

extern "C" {
    pub fn switch_task(current: *mut TaskContext, next: *const TaskContext);
    pub fn switch_to(next: *const TaskContext);
    pub fn first_task_run(next: *const TaskContext);
}
