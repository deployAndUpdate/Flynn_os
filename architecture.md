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
  - `sleep(ticks)`, `block_on_keyboard()`, timer/keyboard wake
  - Shell: `sleep N`, `ps` с Blocked / WAKE_AT
- **Phase 0 — frame allocator**
  - Bitmap по largest usable RAM region (до 1 GiB)
  - `allocate_frame` / `deallocate_frame`, boot self-test (64 cycles)
  - Глобальный allocator (`BootFrameAllocator` → mutex)
  - Shell `mem`: frames total/used/free + heap KiB
  - Стеки задач на mapped frames (`0xFFFF_A000_0000_0000` + slot), не heap

## Будет сделано

### Phase 2.1 — True ISR context switch
- [ ] `iretq` resume из timer ISR

### Phase 5–7 — Processes
- [ ] Page tables per process, Ring 3, `int 0x80` syscall
- [ ] User VA layout, exec, fork/COW, wait

## Целевая модель

| Решение | Выбор | Статус |
|---------|-------|--------|
| Scheduling | Preemptive на timer IRQ | ✅ |
| Ready queue | Приоритетная (+ aging) | ✅ |
| I/O | Block + wake | ✅ |
| Frame allocator | Bitmap + free | ✅ Phase 0 |
| Стек | Mapped frames per task | ✅ Phase 0 |
| Изоляция | Процессы + page tables | Phase 5+ |

## Структура `memory/`

```
kernel/src/memory/
├── layout.rs          — KERNEL_HEAP_*, KERNEL_STACK_*
├── frame_allocator.rs — bitmap, stats, self_test
├── stack.rs           — MappedStack
├── paging.rs          — map_region, with_mapper
├── heap.rs
└── ...
```

## Структура `task/`

```
kernel/src/task/
├── mod.rs
├── context.rs
├── switch.rs
├── preempt.rs
├── scheduler.rs
└── demo.rs
```
