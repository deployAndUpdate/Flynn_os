use heapless::spsc::Queue;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

use crate::interrupts::pic::{InterruptIndex, PICS};

const PS2_STATUS: u16 = 0x64;
const PS2_DATA: u16 = 0x60;
const KEY_BUFFER_SIZE: usize = 64;

lazy_static! {
    static ref KEY_BUFFER: Mutex<Queue<u8, KEY_BUFFER_SIZE>> = Mutex::new(Queue::new());
}

fn wait_ps2_write() {
    let mut status = Port::<u8>::new(PS2_STATUS);
    for _ in 0..100_000 {
        if unsafe { status.read() } & 0x02 == 0 {
            return;
        }
    }
}

/// Enable the first PS/2 port (keyboard). Without this, IRQ1 may never deliver scancodes.
pub fn init_keyboard() {
    wait_ps2_write();
    unsafe {
        Port::<u8>::new(PS2_STATUS).write(0xAE);
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut status_port = Port::<u8>::new(PS2_STATUS);
    let mut data_port = Port::<u8>::new(PS2_DATA);

    loop {
        let status = unsafe { status_port.read() };
        if status & 0x01 == 0 {
            break;
        }
        let scancode = unsafe { data_port.read() };
        let mut buffer = KEY_BUFFER.lock();
        let _ = buffer.enqueue(scancode);
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

pub fn pop_scancode() -> Option<u8> {
    KEY_BUFFER.lock().dequeue()
}
