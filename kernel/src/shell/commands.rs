use crate::driver::serial::SerialPort;

pub fn help() {
    SerialPort::write_str(
        "mem to print the memory map\n\
         ticks to print the number of ticks\n\
         clear to clear the screen\n\
         help to print this help message\n\
         exit to exit the shell\n"
    );
}

pub fn mem() {
    SerialPort::write_str(
        "heap initialized\n"
    );
}

pub fn say(text: &str) {
    SerialPort::write_str(text);
    SerialPort::write_str("\n");
}


pub fn ticks() {
    SerialPort::write_str(
        "ticks\n"
    );
}

pub fn clear() {
    for _ in 0..50 {
        SerialPort::write_str("\n");
    }
}