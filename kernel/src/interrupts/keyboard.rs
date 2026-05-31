use heapless::spsc::Queue;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

use crate::interrupts::pic::{InterruptIndex, PICS};

const KEYBOARD_PORT: u16 = 0x60;
const KEY_BUFFER_SIZE: usize = 64;

lazy_static! {
    static ref KEY_BUFFER: Mutex<Queue<u8, KEY_BUFFER_SIZE>> = Mutex::new(Queue::new());
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::<u8>::new(KEYBOARD_PORT);
    let scancode = unsafe { port.read() };

    let mut buffer = KEY_BUFFER.lock();
    let _ = buffer.enqueue(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

pub fn pop_scancode() -> Option<u8> {
    KEY_BUFFER.lock().dequeue()
}
