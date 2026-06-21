use alloc::string::String;
use spin::Mutex;

use crate::driver::serial::SerialPort;
use crate::input::keyboard::scancode_to_ascii;
use crate::interrupts::keyboard::{flush_scancode_queue, pop_scancode};
use crate::task::PreemptGuard;

const SC_BACKSPACE: u8 = 0x0E;
const SC_ENTER: u8 = 0x1C;
const SC_EXTENDED: u8 = 0xE0;
const CMD_MAX: usize = 128;

static INPUT_LINE: Mutex<String> = Mutex::new(String::new());

/// Show the shell prompt once demo workers are done.
///
/// Clears spurious scancodes and any partial line typed during boot churn.
pub fn show_shell_prompt() {
    flush_scancode_queue();
    INPUT_LINE.lock().clear();
    SerialPort::write_str_no_preempt("\n> ");
}

/// Drain available scancodes. Returns `true` if at least one key was processed.
///
/// `PreemptGuard` is **not** held across shell execution — commands may block (`sleep`).
pub fn process_keyboard_buffer() -> bool {
    let mut processed = false;

    while let Some(scancode) = pop_scancode() {
        processed = true;
        if scancode == SC_EXTENDED {
            continue;
        }
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

/// Drain bytes from COM1 (QEMU `-serial stdio`). Returns `true` if any byte was handled.
pub fn process_serial_buffer() -> bool {
    let mut processed = false;
    while let Some(byte) = SerialPort::try_read_byte() {
        processed = true;
        handle_serial_byte(byte);
    }
    processed
}

fn handle_serial_byte(byte: u8) {
    match byte {
        b'\r' | b'\n' => handle_enter(),
        0x08 | 0x7f => handle_backspace(),
        0x03 => {} // Ctrl+C — ignore (host may send it; do not act on it)
        b if b.is_ascii() && (b.is_ascii_graphic() || b == b' ') => {
            push_char(b as char);
        }
        _ => {}
    }
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
    let mut buf = [0u8; CMD_MAX];
    let len = {
        let _guard = PreemptGuard::new();
        let mut line = INPUT_LINE.lock();
        let trimmed = line.trim();
        let len = trimmed.len().min(CMD_MAX);
        if len > 0 {
            buf[..len].copy_from_slice(&trimmed.as_bytes()[..len]);
        }
        line.clear();
        len
    };

    SerialPort::write_str_no_preempt("\n");
    if len > 0 {
        if let Ok(command) = core::str::from_utf8(&buf[..len]) {
            crate::shell::execute(command);
        }
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
