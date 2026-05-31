use x86_64::instructions::port::Port;

pub fn init_pit() {
    let divisor: u16 = 1193; // ~100 Hz (1.193 MHz base clock)

    unsafe {
        let mut cmd = Port::new(0x43);
        let mut data = Port::new(0x40);

        cmd.write(0x36u8);
        data.write((divisor & 0xFF) as u8);
        data.write((divisor >> 8) as u8);
    }
}
