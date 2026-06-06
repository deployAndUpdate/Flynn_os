# Flynn OS — Architecture Context

Краткий статус эволюции ядра. Обновляется по мере прохождения фаз.

## Сделано

- Bare-metal ядро: boot, serial, paging, heap (1 MiB), PIC, PIT (~100 Hz), keyboard ISR → SPSC queue
- Shell + line input
- **Phase 1** — cooperative kernel threads, lock-free KEY_BUFFER
- **Phase 2** — preemptive timer (flag + safe point), idle task, `PreemptGuard`, `ticks`
- **Phase 3** — priority queue (4 levels) + aging, `ps`, boot dispatch
- **Phase 4 — block / wake**
  - `TaskState::Blocked`, `wake_at` для sleep
  - `sleep(ticks)` — блок до абсолютного tick
  - `block_on_keyboard()` — input ждёт без busy-wait
  - Keyboard ISR → `notify_keyboard_input()` → wake waiter
  - Timer tick → `wake_sleepers(now)`
  - Workers используют `sleep(5)` вместо busy-spin
  - Shell: `sleep N` — тест блокировки
  - `ps` показывает Blocked и WAKE_AT

## Будет сделано

### Phase 2.1 — True ISR context switch
- [ ] `iretq` resume из timer ISR

### Phase 0 — Frame allocator
- [ ] `deallocate_frame`, shell `mem`, mapped stacks

### Phase 5–7 — Processes
- [ ] Page tables, Ring 3, exec, fork/COW, wait

## Целевая модель

| Решение | Выбор | Статус |
|---------|-------|--------|
| Scheduling | Preemptive на timer IRQ | ✅ |
| Ready queue | Приоритетная (+ aging) | ✅ |
| I/O | Block + wake | ✅ Phase 4 |
| Стек | Отдельный на задачу | heap 32 KiB |
| Изоляция | Процессы + page tables | Phase 5+ |

## Структура `task/`

```
kernel/src/task/
├── mod.rs
├── context.rs
├── switch.rs
├── preempt.rs
├── scheduler.rs   — block/wake, sleep, keyboard waiter
└── demo.rs
```
