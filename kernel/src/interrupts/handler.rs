use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;

use crate::interrupts::idt;

lazy_static! {
    pub static ref PICS: Mutex<ChainedPics> =
        Mutex::new(unsafe { ChainedPics::new(32, 40) });
}

pub fn init() {
    idt::init_idt();

    // Remap PIC vectors to 32..47, then mask all IRQ lines.
    // Without masking, the PIT timer fires immediately after `sti` and jumps
    // to an unhandled IDT entry → double/triple fault → reboot loop.
    unsafe {
        PICS.lock().initialize();
        PICS.lock().disable();
    }

    x86_64::instructions::interrupts::enable();
}
