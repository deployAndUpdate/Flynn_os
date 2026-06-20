use crate::alloc::string::ToString;
use alloc::string::String;
use spin::Mutex;

use crate::driver::serial::SerialPort;
use crate::input::keyboard::scancode_to_ascii;
use crate::interrupts::keyboard::pop_scancode;
use crate::task::PreemptGuard;

const SC_BACKSPACE: u8 = 0x0E;
const SC_ENTER: u8 = 0x1C;

static INPUT_LINE: Mutex<String> = Mutex::new(String::new());

/// Drain available scancodes. Returns `true` if at least one key was processed.
///
/// `PreemptGuard` is **not** held across shell execution — commands may block (`sleep`).
pub fn process_keyboard_buffer() -> bool {
    let mut processed = false;

    while let Some(scancode) = pop_scancode() {
        processed = true;
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

    processed
}

fn push_char(ch: char) {
    {
        let _guard = PreemptGuard::new();
        let mut line = INPUT_LINE.lock();
        line.push(ch);
    }
    echo_char(ch);
}

fn handle_enter() {
    let command = {
        let _guard = PreemptGuard::new();
        let mut line = INPUT_LINE.lock();
        let cmd = line.trim().to_string();
        line.clear();
        cmd
    };

    SerialPort::write_str_no_preempt("\n");
    if !command.is_empty() {
        crate::shell::execute(&command);
    }
    SerialPort::write_str_no_preempt("> ");
}

fn handle_backspace() {
    let popped = {
        let _guard = PreemptGuard::new();
        let mut line = INPUT_LINE.lock();
        line.pop()
    };
    if popped.is_some() {
        SerialPort::write_str_no_preempt("\x08 \x08");
    }
}

fn echo_char(ch: char) {
    let mut buf = [0u8; 4];
    SerialPort::write_str_no_preempt(ch.encode_utf8(&mut buf));
}
