use core::cell::UnsafeCell;
use heapless::spsc::Queue;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

use crate::interrupts::pic::{InterruptIndex, PICS};

const PS2_STATUS: u16 = 0x64;
const PS2_DATA: u16 = 0x60;
const KEY_BUFFER_SIZE: usize = 64;

struct KeyQueue(UnsafeCell<Queue<u8, KEY_BUFFER_SIZE>>);

unsafe impl Sync for KeyQueue {}

static KEY_QUEUE: KeyQueue = KeyQueue(UnsafeCell::new(Queue::new()));

/// SAFETY: only the keyboard ISR calls this.
unsafe fn enqueue_scancode(scancode: u8) {
    let queue = &mut *KEY_QUEUE.0.get();
    let _ = queue.enqueue(scancode);
}

/// SAFETY: only the input task / terminal calls this.
pub fn has_scancode() -> bool {
    let queue = unsafe { &*KEY_QUEUE.0.get() };
    !queue.is_empty()
}

/// SAFETY: only `input_loop` / terminal calls this.
pub fn pop_scancode() -> Option<u8> {
    let queue = unsafe { &mut *KEY_QUEUE.0.get() };
    queue.dequeue()
}

/// Drop any scancodes still in the IRQ queue (boot noise, key bounce before prompt).
pub fn flush_scancode_queue() {
    let queue = unsafe { &mut *KEY_QUEUE.0.get() };
    while queue.dequeue().is_some() {}
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
        unsafe { enqueue_scancode(scancode) };
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
