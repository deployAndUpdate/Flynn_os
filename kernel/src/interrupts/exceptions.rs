use core::fmt::Write;

use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

use crate::driver::serial::SerialPort;
use crate::logger::Logger;

pub extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: InterruptStackFrame) {
    panic!("DIVIDE BY ZERO\n{stack_frame:#?}");
}

pub extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    SerialPort::write_str("Breakpoint hit\n");
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let addr = Cr2::read();

    let mut logger = Logger;
    let _ = writeln!(
        logger,
        "PAGE FAULT\naddr: {addr:?}\nerror: {error_code:?}\n{stack_frame:#?}"
    );

    panic!("PAGE FAULT at {addr:?}");
}
