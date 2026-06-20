use crate::driver::serial::SerialPort;
use crate::interrupts::handler;
use crate::memory::{frame_allocator, heap};

pub fn help() {
    SerialPort::write_str(
        "mem to print frame and heap usage\n\
         ticks to print the number of ticks\n\
         ps to list tasks\n\
         sleep N to block for N timer ticks\n\
         clear to clear the screen\n\
         help to print this help message\n"
    );
}

pub fn ps() {
    crate::task::print_ps();
}

pub fn mem() {
    match frame_allocator::stats() {
        Ok(frames) => {
            SerialPort::write_str("frames: total=");
            print_u64(frames.total as u64);
            SerialPort::write_str(" used=");
            print_u64(frames.used as u64);
            SerialPort::write_str(" free=");
            print_u64(frames.free as u64);
            SerialPort::write_str("\n");
        }
        Err(_) => SerialPort::write_str("frames: unavailable\n"),
    }

    let heap = heap::stats();
    SerialPort::write_str("heap:   used=");
    print_u64(bytes_to_kib(heap.used) as u64);
    SerialPort::write_str(" KiB free=");
    print_u64(bytes_to_kib(heap.free) as u64);
    SerialPort::write_str(" KiB total=");
    print_u64(bytes_to_kib(heap.total) as u64);
    SerialPort::write_str(" KiB\n");
}

pub fn say(text: &str) {
    SerialPort::write_str(text);
    SerialPort::write_str("\n");
}

pub fn ticks() {
    SerialPort::write_str("ticks: ");
    print_u64(handler::ticks());
    SerialPort::write_str("\n");
}

pub fn sleep_ticks(ticks: u32) {
    SerialPort::write_str("sleeping ");
    print_u64(ticks as u64);
    SerialPort::write_str(" ticks...\n");
    crate::task::sleep(ticks);
    SerialPort::write_str("woke up\n");
}

pub fn clear() {
    for _ in 0..50 {
        SerialPort::write_str("\n");
    }
}

fn bytes_to_kib(bytes: usize) -> usize {
    bytes / 1024
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
        let byte = [buf[len]];
        SerialPort::write_str(core::str::from_utf8(&byte).unwrap());
    }
}
