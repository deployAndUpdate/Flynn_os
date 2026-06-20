use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

static PREEMPT_DEPTH: AtomicU32 = AtomicU32::new(0);
static ISR_PREEMPT_COUNT: AtomicU64 = AtomicU64::new(0);
/// Running task index — updated before every context switch (visible even if `current` is stale).
static ACTIVE_TASK: AtomicUsize = AtomicUsize::new(usize::MAX);

pub struct PreemptGuard;

impl PreemptGuard {
    pub fn new() -> Self {
        PREEMPT_DEPTH.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for PreemptGuard {
    fn drop(&mut self) {
        PREEMPT_DEPTH.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn is_disabled() -> bool {
    PREEMPT_DEPTH.load(Ordering::SeqCst) > 0
}

pub fn record_isr_preempt() {
    ISR_PREEMPT_COUNT.fetch_add(1, Ordering::SeqCst);
}

pub fn isr_preempt_count() -> u64 {
    ISR_PREEMPT_COUNT.load(Ordering::SeqCst)
}

pub fn set_active_task(idx: usize) {
    ACTIVE_TASK.store(idx, Ordering::SeqCst);
}

pub fn active_task() -> Option<usize> {
    match ACTIVE_TASK.load(Ordering::SeqCst) {
        usize::MAX => None,
        idx => Some(idx),
    }
}
