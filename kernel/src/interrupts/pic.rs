use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    pub static ref PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    });
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

pub fn init_pic() {
    unsafe {
        PICS.lock().initialize();
        PICS.lock().disable();
    }
}

pub fn unmask_timer() {
    unsafe {
        // Unmask IRQ0 (timer) only; keep all other hardware IRQs disabled.
        PICS.lock().write_masks(0xFE, 0xFF);
    }
}
