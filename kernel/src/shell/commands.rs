use crate::driver::serial::SerialPort;
use crate::interrupts::handler;

pub fn help() {
    SerialPort::write_str(
        "mem to print the memory map\n\
         ticks to print the number of ticks\n\
         ps to list tasks\n\
         clear to clear the screen\n\
         help to print this help message\n"
    );
}

pub fn ps() {
    crate::task::print_ps();
}

pub fn mem() {
    SerialPort::write_str("heap initialized\n");
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

pub fn clear() {
    for _ in 0..50 {
        SerialPort::write_str("\n");
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
        let byte = [buf[len]];
        SerialPort::write_str(core::str::from_utf8(&byte).unwrap());
    }
}
