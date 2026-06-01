use alloc::string::String;
use crate::alloc::string::ToString;
use spin::Mutex;

use crate::driver::serial::SerialPort;
use crate::input::keyboard::scancode_to_ascii;
use crate::interrupts::keyboard::pop_scancode;

const SC_BACKSPACE: u8 = 0x0E;
const SC_ENTER: u8 = 0x1C;
pub static INPUT_LINE: Mutex<String> = Mutex::new(String::new());

pub fn push_char(ch: char) {
    let mut line = INPUT_LINE.lock();
    line.push(ch);
    echo_char(ch);
}

pub fn process_keyboard_buffer() {
    while let Some(scancode) = pop_scancode() {
        if scancode & 0x80 != 0 {
            continue;
        }
        match scancode {
            SC_BACKSPACE => handle_backspace(),
            SC_ENTER => handle_enter(),
            code => {
                if let Some(ch) = scancode_to_ascii(code) {
                    push_char(ch);
                }
            }
        }
    }

}

fn handle_enter() {
    let mut line = INPUT_LINE.lock();
    let command = line.trim();
    SerialPort::write_str("\n");
    if !command.is_empty() {
        crate::shell::execute(command);
    }
    line.clear();
    SerialPort::write_str("> ");
}

fn handle_backspace() {
    let mut line = INPUT_LINE.lock();
    if line.pop().is_some() {
        SerialPort::write_str("\x08 \x08");
    }
}

fn echo_char(ch: char) {
    let mut buf = [0u8; 4];
    SerialPort::write_str(ch.encode_utf8(&mut buf));
}

pub fn print_char(ch: char) {
    let my_string: String = ch.to_string();
    let my_str: &str = my_string.as_str();
    SerialPort::write_str(my_str);
}