# Flynn OS — Architecture Context

Краткий статус эволюции ядра. Обновляется по мере прохождения фаз.

## Сделано

- Bare-metal ядро: boot, serial, paging, heap (1 MiB), PIC, PIT (~100 Hz), keyboard ISR → SPSC queue
- Shell + line input
- **Phase 1 — cooperative kernel threads**
  - `task/` модуль: `TaskContext`, `switch_context` (asm), `spawn` / `yield_now` / `start`
  - Отдельный стек на задачу (heap, 16 KiB)
  - FIFO ready queue (priority хранится, пока не используется)
  - `UnsafeCell` scheduler без Mutex (кооперативный режим — lock не переживает context switch)
  - Demo: `worker_a` / `worker_b` чередуются через yield; `input_loop` — shell/клавиатура как задача
  - Проверено в QEMU: `A:0 B:0 A:1 B:1 … A:done B:done`
  - Fix: `KEY_BUFFER` — lock-free SPSC (`UnsafeCell`), без `Mutex` (ISR + task deadlock)

## Будет сделано

### Phase 0 — Frame allocator
- [ ] `deallocate_frame` (free list / bitmap)
- [ ] Shell `mem` — free/used frames
- [ ] Стеки задач на mapped frames вместо heap

### Phase 2 — Preemptive timer
- [ ] Context switch в timer ISR
- [ ] `preempt_disable` / quantum
- [ ] Idle task (`hlt`)
- [ ] Mutex → lock только вне switch; подготовка к SMP

### Phase 3 — Priority queue
- [ ] Multi-level ready queues + aging

### Phase 4 — Block / wake
- [ ] Состояние Blocked, wait queues, `sleep(ticks)`
- [ ] Shell блокируется без busy-wait

### Phase 5–7 — Processes
- [ ] Отдельные page tables, Ring 3
- [ ] exec (ELF), fork/COW, wait

## Целевая модель (зафиксировано)

| Решение | Выбор |
|---------|-------|
| Scheduling | Preemptive на timer IRQ |
| Стек | Отдельный на задачу |
| Ready queue | Приоритетная (+ aging) |
| Switch point | Timer ISR |
| Изоляция | Процессы с отдельными page tables |
| I/O | Block + wake |
| Sync (далёкое) | Per-CPU run queues при SMP |
| Userspace API | POSIX fork/exec/wait |

## Структура `task/`

```
kernel/src/task/
├── mod.rs        — публичный API
├── context.rs    — TaskContext, init stack
├── switch.rs     — switch_context (global_asm)
├── scheduler.rs  — spawn, yield, schedule
└── demo.rs       — worker_a/b, input_loop
```
