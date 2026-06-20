use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::fmt::Write;
use lazy_static::lazy_static;

use crate::driver::serial::SerialPort;
use crate::interrupts::handler;
use crate::logger::Logger;
use crate::memory::stack::MappedStack;
use crate::task::context::{allocate_stack, init_context, TaskContext};
use crate::task::preempt;
use crate::task::switch::{first_task_run, switch_task};

pub type TaskId = u64;

/// Priority levels: 0 (idle) .. MAX_PRIORITY-1 (highest).
pub const MAX_PRIORITY: usize = 4;

/// Timer ticks per time slice (~20 ms at PIT ~100 Hz).
pub const QUANTUM_TICKS: u32 = 2;

/// Ready-queue waits this many timer ticks before boosting one priority level.
const AGING_TICKS: u32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Finished,
}

struct Task {
    id: TaskId,
    state: TaskState,
    priority: u8,
    wait_ticks: u32,
    wake_at: Option<u64>,
    context: TaskContext,
    /// Keeps the mapped stack region alive for the task lifetime.
    #[allow(dead_code)]
    stack: MappedStack,
    entry: fn(),
}

struct Scheduler {
    tasks: Vec<Task>,
    ready: Vec<VecDeque<usize>>,
    current: Option<usize>,
    next_id: TaskId,
    started: bool,
    quantum: u32,
}

struct SchedulerCell(UnsafeCell<Scheduler>);

unsafe impl Sync for SchedulerCell {}

impl Scheduler {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            ready: (0..MAX_PRIORITY).map(|_| VecDeque::new()).collect(),
            current: None,
            next_id: 1,
            started: false,
            quantum: QUANTUM_TICKS,
        }
    }

    fn clamp_priority(priority: u8) -> u8 {
        priority.min((MAX_PRIORITY - 1) as u8)
    }

    fn enqueue_ready(&mut self, idx: usize) {
        let priority = Self::clamp_priority(self.tasks[idx].priority);
        self.tasks[idx].wait_ticks = 0;
        self.tasks[idx].state = TaskState::Ready;
        self.ready[priority as usize].push_back(idx);
    }

    fn wake_task(&mut self, idx: usize) {
        if self.tasks[idx].state != TaskState::Blocked {
            return;
        }
        self.tasks[idx].wake_at = None;
        self.enqueue_ready(idx);
    }

    fn wake_sleepers(&mut self, now: u64) {
        for idx in 0..self.tasks.len() {
            if self.tasks[idx].state != TaskState::Blocked {
                continue;
            }
            if let Some(wake_at) = self.tasks[idx].wake_at {
                if now >= wake_at {
                    self.wake_task(idx);
                }
            }
        }
    }

    fn peek_highest_ready(&self) -> Option<usize> {
        for level in (0..MAX_PRIORITY).rev() {
            for &idx in &self.ready[level] {
                if self.tasks[idx].state == TaskState::Ready {
                    return Some(idx);
                }
            }
        }
        None
    }

    fn pop_highest_ready(&mut self) -> Option<usize> {
        for level in (0..MAX_PRIORITY).rev() {
            while let Some(idx) = self.ready[level].pop_front() {
                if self.tasks[idx].state == TaskState::Ready {
                    return Some(idx);
                }
            }
        }
        None
    }

    fn pop_bootstrap_task(&mut self) -> Option<usize> {
        for level in 1..MAX_PRIORITY {
            while let Some(idx) = self.ready[level].pop_front() {
                if self.tasks[idx].state == TaskState::Ready {
                    return Some(idx);
                }
            }
        }
        self.pop_highest_ready()
    }

    fn age_ready_queues(&mut self) {
        for level in 0..MAX_PRIORITY - 1 {
            let mut i = 0;
            while i < self.ready[level].len() {
                let idx = self.ready[level][i];
                self.tasks[idx].wait_ticks += 1;

                if self.tasks[idx].wait_ticks >= AGING_TICKS {
                    self.tasks[idx].wait_ticks = 0;
                    if let Some(idx) = self.ready[level].remove(i) {
                        self.ready[level + 1].push_back(idx);
                        continue;
                    }
                }
                i += 1;
            }
        }
    }

    fn spawn(&mut self, entry: fn(), priority: u8) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;
        let priority = Self::clamp_priority(priority);

        let mut stack = allocate_stack(id);
        let context = init_context(stack.as_mut_bytes(), task_trampoline);

        let index = self.tasks.len();
        self.tasks.push(Task {
            id,
            state: TaskState::Ready,
            priority,
            wait_ticks: 0,
            wake_at: None,
            context,
            stack,
            entry,
        });
        self.enqueue_ready(index);

        let mut logger = Logger;
        let _ = writeln!(logger, "[task] spawned id={id} prio={priority}");
        id
    }

    fn active_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| matches!(t.state, TaskState::Ready | TaskState::Running))
            .count()
    }

    fn block_current_sleep(&mut self, wake_at: u64) {
        x86_64::instructions::interrupts::disable();
        let idx = self.running_task_index();
        self.tasks[idx].state = TaskState::Blocked;
        self.tasks[idx].wake_at = Some(wake_at);
        self.schedule_next();
    }

    fn yield_current(&mut self) {
        let current_idx = match self.current {
            Some(idx) => idx,
            None => return,
        };

        if self.tasks[current_idx].state == TaskState::Finished {
            self.schedule_next();
            return;
        }

        if self.active_count() <= 1 {
            self.quantum = QUANTUM_TICKS;
            return;
        }

        self.enqueue_ready(current_idx);
        self.quantum = QUANTUM_TICKS;
        self.schedule_next();
    }

    fn finish_current(&mut self) {
        x86_64::instructions::interrupts::disable();
        let idx = self.running_task_index();
        let id = self.tasks[idx].id;
        self.tasks[idx].state = TaskState::Finished;
        let mut logger = Logger;
        let _ = writeln!(logger, "[task] finished id={id}");
        crate::task::demo::try_show_shell_prompt();
        self.schedule_next();
    }

    fn set_running(&mut self, idx: usize) {
        self.current = Some(idx);
        preempt::set_active_task(idx);
    }

    fn running_task_index(&self) -> usize {
        self.current
            .or_else(preempt::active_task)
            .expect("no running task")
    }

    fn schedule_next(&mut self) {
        // Disable IRQ while updating `current` and until the stack switch completes.
        // PreemptGuard cannot be used here — it would stay on a blocked task's stack.
        x86_64::instructions::interrupts::disable();

        let next_idx = if self.started {
            self.pop_highest_ready()
        } else {
            self.pop_bootstrap_task()
        };

        let Some(next_idx) = next_idx else {
            let mut logger = Logger;
            let _ = writeln!(logger, "[task] no runnable tasks — halt");
            loop {
                x86_64::instructions::interrupts::enable();
                x86_64::instructions::hlt();
                x86_64::instructions::interrupts::disable();
            }
        };

        let prev_idx = self.current;

        if prev_idx == Some(next_idx) {
            self.tasks[next_idx].state = TaskState::Running;
            self.tasks[next_idx].wait_ticks = 0;
            x86_64::instructions::interrupts::enable();
            return;
        }

        self.tasks[next_idx].state = TaskState::Running;
        self.tasks[next_idx].wait_ticks = 0;
        self.set_running(next_idx);

        let next_ctx = self.tasks[next_idx].context;

        if let Some(prev) = prev_idx {
            if !self.started {
                panic!("schedule_next: prev task exists before scheduler started");
            }
            let prev_ctx = &mut self.tasks[prev].context;
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            unsafe {
                switch_task(prev_ctx, &next_ctx);
            }
            x86_64::instructions::interrupts::enable();
            return;
        }

        self.started = true;
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        unsafe {
            first_task_run(&next_ctx);
        }
    }

    fn preempt_if_pending(&mut self) {
        if !preempt::take_pending() {
            return;
        }
        if self.active_count() <= 1 {
            return;
        }
        preempt::record_timer_preempt();
        self.yield_current();
    }

    fn on_timer_tick(&mut self, now: u64) {
        self.wake_sleepers(now);

        if self.started && !preempt::is_disabled() {
            self.age_ready_queues();
        }

        if !self.started || preempt::is_disabled() {
            return;
        }
        if self.active_count() <= 1 {
            return;
        }

        if let Some(current_idx) = self.current {
            if let Some(ready_idx) = self.peek_highest_ready() {
                if ready_idx != current_idx
                    && self.tasks[ready_idx].priority > self.tasks[current_idx].priority
                {
                    self.quantum = 0;
                }
            }
        }

        self.quantum = self.quantum.saturating_sub(1);
        if self.quantum == 0 {
            self.quantum = QUANTUM_TICKS;
            preempt::request_preempt();
        }
    }

    fn start(&mut self) -> ! {
        let mut logger = Logger;
        let _ = writeln!(
            logger,
            "[task] phase 2.1: timer preempt (safe points), quantum={} ticks",
            QUANTUM_TICKS
        );
        self.schedule_next();
        loop {
            x86_64::instructions::interrupts::enable();
            x86_64::instructions::hlt();
        }
    }

    fn current_entry(&self) -> fn() {
        let idx = self.running_task_index();
        self.tasks[idx].entry
    }

    fn print_ps(&self) {
        let _guard = crate::task::PreemptGuard::new();
        SerialPort::write_str("ID  PRIO  STATE     WAIT  WAKE_AT\n");
        for task in &self.tasks {
            SerialPort::write_str(" ");
            print_u64(task.id);
            SerialPort::write_str("   ");
            print_u64(task.priority as u64);
            SerialPort::write_str("     ");
            match task.state {
                TaskState::Ready => SerialPort::write_str("Ready   "),
                TaskState::Running => SerialPort::write_str("Running "),
                TaskState::Blocked => SerialPort::write_str("Blocked "),
                TaskState::Finished => SerialPort::write_str("Finished"),
            }
            SerialPort::write_str("  ");
            print_u64(task.wait_ticks as u64);
            SerialPort::write_str("   ");
            match task.wake_at {
                Some(t) => print_u64(t),
                None => SerialPort::write_str("-"),
            }
            SerialPort::write_str("\n");
        }
    }
}

fn print_u64(mut n: u64) {
    if n == 0 {
        SerialPort::write_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut len = 0;
    while n > 0 {
        buf[len] = b'0' + (n % 10) as u8;
        len += 1;
        n /= 10;
    }
    while len > 0 {
        len -= 1;
        SerialPort::write_str(core::str::from_utf8(&[buf[len]]).unwrap());
    }
}

lazy_static! {
    static ref SCHEDULER: SchedulerCell = SchedulerCell(UnsafeCell::new(Scheduler::new()));
}

fn scheduler() -> &'static mut Scheduler {
    unsafe { &mut *SCHEDULER.0.get() }
}

pub fn spawn(entry: fn(), priority: u8) -> TaskId {
    scheduler().spawn(entry, priority)
}

#[allow(dead_code)]
pub fn yield_now() {
    scheduler().yield_current();
}

pub fn start() -> ! {
    scheduler().start();
}

pub fn sleep(ticks: u32) {
    let now = handler::ticks();
    let wake_at = now + ticks as u64;
    scheduler().block_current_sleep(wake_at);
}

pub fn on_timer_tick() {
    let now = handler::ticks();
    scheduler().on_timer_tick(now);
}

pub fn preempt_if_pending() {
    scheduler().preempt_if_pending();
}

pub fn isr_preempt_count() -> u64 {
    preempt::isr_preempt_count()
}

pub fn print_ps() {
    scheduler().print_ps();
}

extern "C" fn task_trampoline() -> ! {
    let entry = scheduler().current_entry();
    entry();
    scheduler().finish_current();
    loop {
        x86_64::instructions::interrupts::enable();
        x86_64::instructions::hlt();
    }
}
