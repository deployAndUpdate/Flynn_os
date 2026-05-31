use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

use crate::interrupts::exceptions::{
    breakpoint_handler, divide_by_zero_handler, page_fault_handler,
};
use crate::interrupts::handler::timer_interrupt_handler;
use crate::interrupts::keyboard::keyboard_interrupt_handler;
use crate::interrupts::pic::InterruptIndex;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.divide_error.set_handler_fn(divide_by_zero_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}
