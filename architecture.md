# Flynn OS — Architecture Context

Краткий статус эволюции ядра. Обновляется по мере прохождения фаз.

## Сделано

- Bare-metal ядро: boot, serial, paging, heap (1 MiB), PIC, PIT (~100 Hz), keyboard ISR → SPSC queue
- Shell + line input
- **Phase 1** — cooperative kernel threads, lock-free KEY_BUFFER
- **Phase 2** — preemptive timer (flag + safe point), idle task, `PreemptGuard`, `ticks` command
- **Phase 3 — priority queue + aging**
  - 4 уровня приоритета (0=idle .. 3=max)
  - `ready[level]` — отдельная FIFO на уровень
  - `pop_highest_ready()` — выбор max prio
  - Aging: каждые 15 timer ticks задача в ready повышается на 1 уровень (cap)
  - Shell `ps` — id, prio, state, wait_ticks
  - Приоритеты: workers=1, input=2, idle=0
  - Boot: `pop_bootstrap_task()` — старт с prio 1+, затем `pop_highest_ready()`
  - Skip yield/switch при одной runnable задаче
  - Стек задачи: 32 KiB

## Будет сделано

### Phase 2.1 — True ISR context switch
- [ ] `iretq` resume из timer ISR

### Phase 0 — Frame allocator
- [ ] `deallocate_frame`, shell `mem`, mapped stacks

### Phase 4 — Block / wake
- [ ] Blocked, wait queues, `sleep(ticks)`

### Phase 5–7 — Processes
- [ ] Page tables, Ring 3, exec, fork/COW, wait

## Целевая модель

| Решение | Выбор | Статус |
|---------|-------|--------|
| Scheduling | Preemptive на timer IRQ | ✅ Phase 2 |
| Ready queue | Приоритетная (+ aging) | ✅ Phase 3 |
| Стек | Отдельный на задачу | heap 16 KiB |
| Switch point | Timer ISR → flag → task | ✅ |
| Изоляция | Процессы + page tables | Phase 5+ |

## Структура `task/`

```
kernel/src/task/
├── mod.rs
├── context.rs
├── switch.rs
├── preempt.rs
├── scheduler.rs   — multi-level ready + aging
└── demo.rs
```
