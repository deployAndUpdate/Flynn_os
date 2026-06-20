use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(
    r#"
    .global switch_task
    .global switch_to
    .global first_task_run

    // Voluntary slot: [rsp] = return address → synthetic iretq frame (IF=1 atomically).
    .macro voluntary_iretq
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
    .endm

    // void switch_task(TaskContext* current, TaskContext* next)
    switch_task:
        mov [rdi], rsp
        mov byte ptr [rdi + 8], 0
        mov rax, [rsi]
        mov rsp, rax
        cmp byte ptr [rsi + 8], 0
        je 1f
        iretq
    1:  voluntary_iretq

    // void switch_to(TaskContext* next) — called from timer ISR; must iretq into task.
    switch_to:
        mov rax, [rdi]
        mov rsp, rax
        cmp byte ptr [rdi + 8], 0
        je 2f
        iretq
    2:  voluntary_iretq

    // void first_task_run(TaskContext* next) — bootstrap from kernel.
    first_task_run:
        mov rax, [rdi]
        mov rsp, rax
        cmp byte ptr [rdi + 8], 0
        je 3f
        iretq
    3:  voluntary_iretq
    "#
);

extern "C" {
    pub fn switch_task(current: *mut TaskContext, next: *const TaskContext);
    pub fn switch_to(next: *const TaskContext);
    pub fn first_task_run(next: *const TaskContext);
}
