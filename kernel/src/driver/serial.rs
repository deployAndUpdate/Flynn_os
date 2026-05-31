use x86_64::instructions::port::Port;

const COM1: u16 = 0x3F8;

pub struct SerialPort;

impl SerialPort {
    pub fn init() {
        unsafe {
            let mut port = Port::<u8>::new(COM1 + 1);
            port.write(0x00); // disable interrupts

            let mut port = Port::<u8>::new(COM1 + 3);
            port.write(0x80); // enable DLAB

            let mut port = Port::<u8>::new(COM1 + 0);
            port.write(0x03); // baud divisor low (38400)

            let mut port = Port::<u8>::new(COM1 + 1);
            port.write(0x00); // baud divisor high

            let mut port = Port::<u8>::new(COM1 + 3);
            port.write(0x03); // 8 bits, no parity, one stop bit

            let mut port = Port::<u8>::new(COM1 + 2);
            port.write(0xC7); // FIFO

            let mut port = Port::<u8>::new(COM1 + 4);
            port.write(0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    fn send(byte: u8) {
        unsafe {
            let mut port = Port::<u8>::new(COM1);
            port.write(byte);
        }
    }

    pub fn write_str(s: &str) {
        for b in s.bytes() {
            Self::send(b);
        }
    }
}
