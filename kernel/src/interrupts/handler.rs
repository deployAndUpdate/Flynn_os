use x86_64::structures::idt::InterruptStackFrame;

use crate::driver::serial::SerialPort;
use crate::interrupts::idt;
use crate::interrupts::pic::{InterruptIndex, PICS};
use crate::interrupts::pit;

static mut TICKS: u64 = 0;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        TICKS += 1;

        // if TICKS % 100 == 0 {
        //     SerialPort::write_str("tick\n");
        // }

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

pub fn init() {
    init_interrupts();
}

pub fn init_interrupts() {
    crate::interrupts::pic::init_pic();
    pit::init_pit();
    crate::interrupts::keyboard::init_keyboard();
    idt::init_idt();
    crate::interrupts::pic::unmask_irqs();
    x86_64::instructions::interrupts::enable();
}

pub fn ticks() -> u64 {
    unsafe { TICKS }
}
