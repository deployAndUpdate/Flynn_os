# Flynn OS — Architecture Context

Краткий статус эволюции ядра. Обновляется по мере прохождения фаз.

## Сделано

- Bare-metal ядро: boot, serial, paging, heap (1 MiB), PIC, PIT (~100 Hz), keyboard ISR → SPSC queue
- Shell + line input
- **Phase 1 — cooperative kernel threads**
  - `task/`: `TaskContext`, `switch_task`, `spawn` / `yield_now` / `start`
  - Fix: `KEY_BUFFER` lock-free SPSC
- **Phase 2 — preemptive timer**
  - Timer ISR: EOI → `on_timer_tick()` → `preempt::request()` при quantum=0
  - Quantum = 2 ticks (~20 ms), `PreemptGuard` / `preempt_disable`
  - `preempt_if_pending()` в safe points (busy_spin, idle, input)
  - Idle task (`hlt`, prio 0)
  - Workers без явного yield — preempt по флагу от timer
  - `ticks` shell command
  - **Примечание:** switch в task context (не `iretq` из ISR) — `iretq`-preempt отложен

## Будет сделано

### Phase 2.1 — True ISR context switch
- [ ] `iretq` resume из timer ISR (сохранение реального interrupt frame)

### Phase 0 — Frame allocator
- [ ] `deallocate_frame`, shell `mem`, mapped stacks

### Phase 3 — Priority queue
- [ ] Multi-level ready queues + aging

### Phase 4 — Block / wake
- [ ] Blocked, wait queues, `sleep(ticks)`

### Phase 5–7 — Processes
- [ ] Page tables, Ring 3, exec, fork/COW, wait

## Целевая модель

| Решение | Выбор | Статус |
|---------|-------|--------|
| Scheduling | Preemptive на timer IRQ | Phase 2 (flag + safe point) |
| Стек | Отдельный на задачу | heap 16 KiB |
| Ready queue | Приоритетная (+ aging) | FIFO (Phase 3) |
| Switch point | Timer ISR | request в ISR, switch в task |
| Изоляция | Процессы + page tables | Phase 5+ |

## Структура `task/`

```
kernel/src/task/
├── mod.rs
├── context.rs
├── switch.rs       — switch_task (ret)
├── preempt.rs      — PreemptGuard, pending flag
├── scheduler.rs
└── demo.rs
```
