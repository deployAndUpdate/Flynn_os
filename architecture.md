# Flynn OS — Architecture Context

Краткий статус эволюции ядра. Обновляется по мере прохождения фаз.

## Сделано

- Bare-metal ядро: boot, serial, paging, heap (1 MiB), PIC, PIT (~100 Hz), keyboard ISR → SPSC queue
- Shell + line input
- **Phase 1** — cooperative kernel threads, lock-free KEY_BUFFER
- **Phase 2** — preemptive timer, idle task, `PreemptGuard`, `ticks`
- **Phase 2.1 — Timer preempt at safe points**
  - Timer ISR sets `PREEMPT_PENDING`; switch runs in task context (`burn`, `idle`)
  - Voluntary `switch_task` + synthetic `iretq` (IF=1)
  - Shell: `preempts` — timer-driven preemption count
  - True ISR `iretq` switch deferred (needs full GPR save)
- **Phase 3** — priority queue (4 levels) + aging, `ps`, boot dispatch
- **Phase 4** — block / wake (`sleep`, `block_on_keyboard`)
- **Phase 0** — bitmap frame allocator, `deallocate_frame`, shell `mem`, mapped stacks

## Будет сделано

### Phase 5–7 — Processes
- [ ] Page tables per process, Ring 3, `int 0x80` syscall
- [ ] User VA layout, exec, fork/COW, wait

## Целевая модель

| Решение | Выбор | Статус |
|---------|-------|--------|
| Scheduling | Preemptive на timer IRQ (`iretq`) | ✅ Phase 2.1 |
| Ready queue | Приоритетная (+ aging) | ✅ |
| I/O | Block + wake | ✅ |
| Frame allocator | Bitmap + free | ✅ |
| Стек | Mapped frames per task | ✅ |
| Изоляция | Процессы + page tables | Phase 5+ |

## Структура `task/`

```
kernel/src/task/
├── context.rs     — TaskContext, save_preempt_frame
├── switch.rs      — switch_task, first_task_run (asm)
├── preempt.rs     — PreemptGuard, isr_preempt_count
├── scheduler.rs   — preempt_from_interrupt
└── demo.rs
```
