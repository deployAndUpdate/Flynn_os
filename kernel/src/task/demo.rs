use crate::driver::serial::SerialPort;
use crate::input::terminal;
use crate::task::preempt_if_pending;

const WORKER_ITERATIONS: u32 = 5;

/// Busy-loop worker — no voluntary yield; timer IRQ sets preempt flag.
pub fn worker_a() {
    for i in 0..WORKER_ITERATIONS {
        SerialPort::write_str("A:");
        print_u32(i);
        SerialPort::write_str("\n");
        busy_spin();
    }
    SerialPort::write_str("A:done\n");
}

pub fn worker_b() {
    for i in 0..WORKER_ITERATIONS {
        SerialPort::write_str("B:");
        print_u32(i);
        SerialPort::write_str("\n");
        busy_spin();
    }
    SerialPort::write_str("B:done\n");
}

pub fn input_loop() {
    loop {
        terminal::process_keyboard_buffer();
        preempt_if_pending();
        crate::task::yield_now();
    }
}

pub fn idle() {
    loop {
        preempt_if_pending();
        x86_64::instructions::hlt();
    }
}

fn busy_spin() {
    for _ in 0..50_000 {
        preempt_if_pending();
        core::hint::spin_loop();
    }
}

fn print_u32(mut n: u32) {
    if n == 0 {
        SerialPort::write_str("0");
        return;
    }

    let mut buf = [0u8; 10];
    let mut len = 0;
    while n > 0 {
        buf[len] = b'0' + (n % 10) as u8;
        len += 1;
        n /= 10;
    }

    while len > 0 {
        len -= 1;
        let mut byte = [0u8; 1];
        byte[0] = buf[len];
        SerialPort::write_str(core::str::from_utf8(&byte).unwrap());
    }
}
