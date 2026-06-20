use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::driver::serial::SerialPort;
use crate::input::terminal;
use crate::interrupts::keyboard::has_scancode;
use crate::task::{block_on_keyboard, isr_preempt_count, preempt_if_pending, sleep};

const WORKER_ITERATIONS: u32 = 5;

static WORKERS_FINISHED: AtomicU32 = AtomicU32::new(0);
static SHELL_PROMPT_SHOWN: AtomicBool = AtomicBool::new(false);

pub fn note_worker_finished() {
    WORKERS_FINISHED.fetch_add(1, Ordering::SeqCst);
}

/// Called from `finish_current` after the "[task] finished" line.
pub fn try_show_shell_prompt() {
    if WORKERS_FINISHED.load(Ordering::SeqCst) < 2 {
        return;
    }
    if SHELL_PROMPT_SHOWN.swap(true, Ordering::SeqCst) {
        return;
    }

    terminal::show_shell_prompt();

    let mut buf = [0u8; 48];
    let prefix = b"[task] isr_preempts=";
    buf[..prefix.len()].copy_from_slice(prefix);
    let n = format_u64(isr_preempt_count(), &mut buf[prefix.len()..]);
    let end = prefix.len() + n;
    buf[end] = b'\n';
    SerialPort::write_str_no_preempt(core::str::from_utf8(&buf[..=end]).unwrap());
}

/// Burn CPU so timer quantum expires mid-work (preempt test).
fn burn() {
    for _ in 0..2_000_000 {
        preempt_if_pending();
        core::hint::spin_loop();
    }
}

fn log_worker_line(name: char, i: u32) {
    let _guard = crate::task::PreemptGuard::new();
    let mut buf = [0u8; 12];
    buf[0] = name as u8;
    buf[1] = b':';
    let n = format_u32(i, &mut buf[2..]);
    let end = 2 + n;
    buf[end] = b'\n';
    SerialPort::write_str(core::str::from_utf8(&buf[..=end]).unwrap());
}

pub fn worker_a() {
    for i in 0..WORKER_ITERATIONS {
        log_worker_line('A', i);
        burn();
        sleep(5);
    }
    SerialPort::write_str_no_preempt("A:done\n");
    note_worker_finished();
}

pub fn worker_b() {
    for i in 0..WORKER_ITERATIONS {
        log_worker_line('B', i);
        burn();
        sleep(5);
    }
    SerialPort::write_str_no_preempt("B:done\n");
    note_worker_finished();
}

/// Blocks when no keyboard input — no busy-wait polling.
pub fn input_loop() {
    loop {
        if has_scancode() {
            terminal::process_keyboard_buffer();
        } else {
            block_on_keyboard();
        }
    }
}

pub fn idle() {
    loop {
        x86_64::instructions::interrupts::enable();
        preempt_if_pending();
        x86_64::instructions::hlt();
    }
}

fn format_u32(mut n: u32, out: &mut [u8]) -> usize {
    if n == 0 {
        out[0] = b'0';
        return 1;
    }

    let mut tmp = [0u8; 10];
    let mut len = 0;
    while n > 0 {
        tmp[len] = b'0' + (n % 10) as u8;
        len += 1;
        n /= 10;
    }

    for i in 0..len {
        out[i] = tmp[len - 1 - i];
    }
    len
}

fn format_u64(mut n: u64, out: &mut [u8]) -> usize {
    if n == 0 {
        out[0] = b'0';
        return 1;
    }

    let mut tmp = [0u8; 20];
    let mut len = 0;
    while n > 0 {
        tmp[len] = b'0' + (n % 10) as u8;
        len += 1;
        n /= 10;
    }

    for i in 0..len {
        out[i] = tmp[len - 1 - i];
    }
    len
}
