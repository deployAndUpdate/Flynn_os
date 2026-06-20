use crate::driver::serial::SerialPort;
use crate::interrupts::handler;
use crate::memory::{frame_allocator, heap};

pub fn help() {
    SerialPort::write_str_no_preempt(
        "mem to print frame and heap usage\n\
         preempts to print ISR preemption count\n\
         ticks to print the number of ticks\n\
         ps to list tasks\n\
         sleep N to block for N timer ticks\n\
         clear to clear the screen\n\
         help to print this help message\n",
    );
}

pub fn ps() {
    crate::task::print_ps();
}

pub fn preempts() {
    SerialPort::write_str_no_preempt("isr_preempts: ");
    print_u64(crate::task::isr_preempt_count());
    SerialPort::write_str_no_preempt("\n");
}

pub fn mem() {
    match frame_allocator::stats() {
        Ok(frames) => {
            SerialPort::write_str_no_preempt("frames: total=");
            print_u64(frames.total as u64);
            SerialPort::write_str_no_preempt(" used=");
            print_u64(frames.used as u64);
            SerialPort::write_str_no_preempt(" free=");
            print_u64(frames.free as u64);
            SerialPort::write_str_no_preempt("\n");
        }
        Err(_) => SerialPort::write_str_no_preempt("frames: unavailable\n"),
    }

    let heap = heap::stats();
    SerialPort::write_str_no_preempt("heap:   used=");
    print_u64(bytes_to_kib(heap.used) as u64);
    SerialPort::write_str_no_preempt(" KiB free=");
    print_u64(bytes_to_kib(heap.free) as u64);
    SerialPort::write_str_no_preempt(" KiB total=");
    print_u64(bytes_to_kib(heap.total) as u64);
    SerialPort::write_str_no_preempt(" KiB\n");
}

pub fn say(text: &str) {
    SerialPort::write_str_no_preempt(text);
    SerialPort::write_str_no_preempt("\n");
}

pub fn ticks() {
    SerialPort::write_str_no_preempt("ticks: ");
    print_u64(handler::ticks());
    SerialPort::write_str_no_preempt("\n");
}

pub fn sleep_ticks(ticks: u32) {
    SerialPort::write_str_no_preempt("sleeping ");
    print_u64(ticks as u64);
    SerialPort::write_str_no_preempt(" ticks...\n");
    crate::task::sleep(ticks);
    SerialPort::write_str_no_preempt("woke up\n");
}

pub fn clear() {
    for _ in 0..50 {
        SerialPort::write_str_no_preempt("\n");
    }
}

fn bytes_to_kib(bytes: usize) -> usize {
    bytes / 1024
}

fn print_u64(mut n: u64) {
    if n == 0 {
        SerialPort::write_str_no_preempt("0");
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
        SerialPort::write_str_no_preempt(core::str::from_utf8(&byte).unwrap());
    }
}
