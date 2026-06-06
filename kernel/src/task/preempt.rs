use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static PREEMPT_DEPTH: AtomicU32 = AtomicU32::new(0);
static PREEMPT_PENDING: AtomicBool = AtomicBool::new(false);

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

/// Set by timer ISR when quantum expires.
pub fn request() {
    PREEMPT_PENDING.store(true, Ordering::SeqCst);
}

/// Checked by tasks at safe points; clears the flag.
pub fn take_pending() -> bool {
    PREEMPT_PENDING.swap(false, Ordering::SeqCst)
}
