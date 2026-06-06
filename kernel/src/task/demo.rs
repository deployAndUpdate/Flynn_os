use crate::driver::serial::SerialPort;
use crate::input::terminal;
use crate::interrupts::keyboard::has_scancode;
use crate::task::{block_on_keyboard, preempt_if_pending, sleep};

const WORKER_ITERATIONS: u32 = 5;

pub fn worker_a() {
    for i in 0..WORKER_ITERATIONS {
        SerialPort::write_str("A:");
        print_u32(i);
        SerialPort::write_str("\n");
        sleep(5);
    }
    SerialPort::write_str("A:done\n");
}

pub fn worker_b() {
    for i in 0..WORKER_ITERATIONS {
        SerialPort::write_str("B:");
        print_u32(i);
        SerialPort::write_str("\n");
        sleep(5);
    }
    SerialPort::write_str("B:done\n");
}

/// Blocks when no keyboard input — no busy-wait polling.
pub fn input_loop() {
    loop {
        if has_scancode() {
            terminal::process_keyboard_buffer();
        } else {
            block_on_keyboard();
        }
        preempt_if_pending();
    }
}

pub fn idle() {
    loop {
        preempt_if_pending();
        x86_64::instructions::hlt();
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
