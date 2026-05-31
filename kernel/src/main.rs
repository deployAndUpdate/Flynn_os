#![no_std]
#![no_main]

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;
use core::panic::PanicInfo;
use uart_16550::SerialPort;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut serial = serial();
    writeln!(serial, "Hi from kernel (serial)").ok();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        draw_hi_marker(framebuffer);
        writeln!(serial, "Framebuffer: {}x{}", framebuffer.info().width, framebuffer.info().height).ok();
    } else {
        writeln!(serial, "No framebuffer").ok();
    }

    loop {}
}

/// White block in the top-left — visible proof that the framebuffer works.
fn draw_hi_marker(framebuffer: &mut FrameBuffer) {
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();

    for y in 0..40 {
        for x in 0..80 {
            write_pixel(buffer, &info, x, y, 0xff, 0xff, 0xff);
        }
    }
}

fn write_pixel(buffer: &mut [u8], info: &FrameBufferInfo, x: usize, y: usize, r: u8, g: u8, b: u8) {
    if x >= info.width || y >= info.height {
        return;
    }

    let offset = y * info.stride * info.bytes_per_pixel + x * info.bytes_per_pixel;
    match info.pixel_format {
        PixelFormat::Bgr => {
            buffer[offset] = b;
            buffer[offset + 1] = g;
            buffer[offset + 2] = r;
        }
        PixelFormat::Rgb => {
            buffer[offset] = r;
            buffer[offset + 1] = g;
            buffer[offset + 2] = b;
        }
        _ => {}
    }
}

fn serial() -> SerialPort {
    let mut port = unsafe { SerialPort::new(0x3F8) };
    port.init();
    port
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = serial();
    let _ = writeln!(serial, "PANIC: {info}");
    loop {}
}
