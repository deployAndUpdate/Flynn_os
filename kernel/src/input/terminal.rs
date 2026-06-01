use alloc::string::String;
use spin::Mutex;

use crate::driver::serial::SerialPort;
use crate::input::keyboard::scancode_to_ascii;
use crate::interrupts::keyboard::pop_scancode;

pub static INPUT_LINE: Mutex<String> = Mutex::new(String::new());

pub fn push_char(ch: char) {
    let mut line = INPUT_LINE.lock();

    if ch == '\n' {
        SerialPort::write_str("\n[cmd] ");
        SerialPort::write_str(line.as_str());
        SerialPort::write_str("\n");
        line.clear();
        return;
    }

    line.push(ch);
}

pub fn process_keyboard_buffer() {
    while let Some(scancode) = pop_scancode() {
        if scancode & 0x80 != 0 {
            continue;
        }
        if let Some(ch) = scancode_to_ascii(scancode) {
            push_char(ch);
        }
    }
}
