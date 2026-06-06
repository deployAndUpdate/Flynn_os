use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::fmt::Write;
use lazy_static::lazy_static;

use crate::logger::Logger;
use crate::task::context::{allocate_stack, init_context, TaskContext};
use crate::task::switch::switch_context;

pub type TaskId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Finished,
}

struct Task {
    id: TaskId,
    state: TaskState,
    priority: u8,
    context: TaskContext,
    stack: Vec<u8>,
    entry: fn(),
}

struct Scheduler {
    tasks: Vec<Task>,
    ready: VecDeque<usize>,
    current: Option<usize>,
    next_id: TaskId,
    bootstrap: TaskContext,
    started: bool,
}

struct SchedulerCell(UnsafeCell<Scheduler>);

unsafe impl Sync for SchedulerCell {}

impl Scheduler {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            ready: VecDeque::new(),
            current: None,
            next_id: 1,
            bootstrap: TaskContext { rsp: 0 },
            started: false,
        }
    }

    fn spawn(&mut self, entry: fn(), priority: u8) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;

        let mut stack = allocate_stack();
        let context = init_context(&mut stack, task_trampoline);

        let index = self.tasks.len();
        self.tasks.push(Task {
            id,
            state: TaskState::Ready,
            priority,
            context,
            stack,
            entry,
        });
        self.ready.push_back(index);

        let mut logger = Logger;
        let _ = writeln!(logger, "[task] spawned id={id} prio={priority}");
        id
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

        self.tasks[current_idx].state = TaskState::Ready;
        self.ready.push_back(current_idx);
        self.schedule_next();
    }

    fn finish_current(&mut self) {
        if let Some(idx) = self.current {
            let id = self.tasks[idx].id;
            self.tasks[idx].state = TaskState::Finished;
            let mut logger = Logger;
            let _ = writeln!(logger, "[task] finished id={id}");
        }
        self.schedule_next();
    }

    fn schedule_next(&mut self) {
        while let Some(next_idx) = self.ready.pop_front() {
            if self.tasks[next_idx].state == TaskState::Finished {
                continue;
            }

            let prev_idx = self.current;
            self.tasks[next_idx].state = TaskState::Running;
            self.current = Some(next_idx);

            let next_ctx = self.tasks[next_idx].context;

            if let Some(prev_idx) = prev_idx {
                if !self.started {
                    panic!("schedule_next: prev task exists before scheduler started");
                }
                let prev_ctx = &mut self.tasks[prev_idx].context;
                unsafe {
                    switch_context(prev_ctx, &next_ctx);
                }
                return;
            }

            self.started = true;
            unsafe {
                switch_context(&mut self.bootstrap, &next_ctx);
            }
            return;
        }

        let mut logger = Logger;
        let _ = writeln!(logger, "[task] no runnable tasks — halt");
        loop {
            x86_64::instructions::hlt();
        }
    }

    fn start(&mut self) -> ! {
        let mut logger = Logger;
        let _ = writeln!(
            logger,
            "[task] starting cooperative scheduler ({} tasks)",
            self.tasks.len()
        );
        self.schedule_next();
        loop {
            x86_64::instructions::hlt();
        }
    }

    fn current_entry(&self) -> fn() {
        let idx = self.current.expect("trampoline without current task");
        self.tasks[idx].entry
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

pub fn yield_now() {
    scheduler().yield_current();
}

pub fn start() -> ! {
    scheduler().start();
}

extern "C" fn task_trampoline() -> ! {
    let entry = scheduler().current_entry();
    entry();
    scheduler().finish_current();
    loop {
        x86_64::instructions::hlt();
    }
}
