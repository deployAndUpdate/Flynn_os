use core::fmt;
use crate::driver::serial::SerialPort;

pub struct Logger;

impl Logger {
    pub fn init() {
        SerialPort::init();
        SerialPort::write_str("[boot] logger initialized\n");
    }
}

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        SerialPort::write_str(s);
        Ok(())
    }
}
