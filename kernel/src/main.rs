#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod driver;
mod input;
mod interrupts;
mod logger;
mod memory;
mod shell;
mod task;

use alloc::vec::Vec;
use bootloader_api::config::Mapping;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::fmt::Write;
use core::panic::PanicInfo;
use logger::Logger;
use x86_64::structures::paging::FrameAllocator;
use x86_64::VirtAddr;

use memory::layout::{KERNEL_HEAP_SIZE, KERNEL_HEAP_START};

const HEAP_START: usize = KERNEL_HEAP_START;
const HEAP_SIZE: usize = KERNEL_HEAP_SIZE;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    Logger::init();

    let mut logger = Logger;
    writeln!(logger, "[boot] memory map received").ok();
    memory::memory_map::print_memory_map(&boot_info.memory_regions);

    writeln!(logger, "[mem] initializing frame allocator").ok();
    let mut frame_allocator = memory::frame_allocator::init_frame_allocator(boot_info);
    writeln!(logger, "[mem] frame allocator ready").ok();
    match frame_allocator.allocate_frame() {
        Some(frame) => {
            writeln!(
                logger,
                "[mem] probe frame at {:#018x}",
                frame.start_address().as_u64()
            )
            .ok();
        }
        None => {
            writeln!(logger, "[mem] no frames available").ok();
        }
    }

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("bootloader must map physical memory"),
    );
    memory::paging::init_phys_mem_offset(phys_mem_offset);
    let mut mapper = unsafe { memory::paging::active_page_table(phys_mem_offset) };
    writeln!(
        logger,
        "[paging] active page table initialized (phys offset {phys_mem_offset:#x})"
    )
    .ok();

    memory::paging::map_heap(&mut mapper, &mut frame_allocator, HEAP_START, HEAP_SIZE);
    memory::heap::init_heap(HEAP_START, HEAP_SIZE);
    writeln!(
        logger,
        "[heap] initialized at {HEAP_START:#018x} ({HEAP_SIZE} bytes)"
    )
    .ok();

    match memory::frame_allocator::self_test(64) {
        Ok(n) => {
            writeln!(logger, "[mem] frame self-test ok ({n} alloc/free cycles)").ok();
        }
        Err(e) => {
            writeln!(logger, "[mem] frame self-test FAILED: {e:?}").ok();
        }
    }

    if let Ok(stats) = memory::frame_allocator::stats() {
        writeln!(
            logger,
            "[mem] frames: total={} used={} free={}",
            stats.total, stats.used, stats.free
        )
        .ok();
    }

    let v = Vec::from([1, 2]);
    writeln!(
        logger,
        "[heap] Vec works (len={}, sum={})",
        v.len(),
        v.iter().sum::<i32>()
    )
    .ok();

    interrupts::handler::init();
    writeln!(logger, "[interrupts] timer + keyboard IRQ enabled").ok();

    show_banner();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        draw_hi_marker(framebuffer);
        writeln!(
            logger,
            "Framebuffer: {}x{}",
            framebuffer.info().width,
            framebuffer.info().height
        )
        .ok();
    } else {
        writeln!(logger, "No framebuffer").ok();
    }

    writeln!(logger, "[task] phase 0: frame bitmap + mapped stacks").ok();

    task::spawn(task::demo::worker_a, 1);
    task::spawn(task::demo::worker_b, 1);
    task::spawn(task::demo::input_loop, 2);
    task::spawn(task::demo::idle, 0);

    crate::driver::serial::SerialPort::write_str("> ");
    writeln!(logger, "Hi from kernel (serial)").ok();

    task::start();
}

fn show_banner() {
    let mut logger = Logger;
    const BANNER: &str = r"
                     .-=================-.
                 .-==#%%%%%%%%%%%%%%%%%%%#==-.
              .-==#%%%%%%%%%%%%%%%%%%%%%%%%%%%#==-.
            .-=#%%%%%%%%%%%%%%#***#%%%%%%%%%%%%%%#=-.
           :=#%%%%%%%%%%%%#*=:.   .:=*#%%%%%%%%%%%%#=:
         .-*%%%%%%%%%%%#*=.           .=*#%%%%%%%%%%%*-.
        .=#%%%%%%%%%%#=.                 .=#%%%%%%%%%%#=.
        :#%%%%%%%%%%#:       FLYNN        :#%%%%%%%%%%#:
        :#%%%%%%%%%%#:         OS         :#%%%%%%%%%%#:
        .=#%%%%%%%%%%#=.               .=#%%%%%%%%%%#=.
         .-*%%%%%%%%%%%#*=.         .=*#%%%%%%%%%%%*-.
           :=#%%%%%%%%%%%%#*=-...-=*#%%%%%%%%%%%%#=:
            .-=#%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%#=-.
               .-==#%%%%%%%%%%%%%%%%%%%%%%%#==-.
                    '-=================-'

             [ GRID LINK ESTABLISHED ]
             [ KERNEL STATUS : ONLINE ]
             [ USER SPACE    : ACTIVE ]
";
    write!(logger, "{BANNER}").ok();
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut logger = Logger;
    let _ = writeln!(logger, "PANIC: {info}");
    loop {}
}
