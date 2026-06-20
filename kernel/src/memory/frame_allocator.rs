use bootloader_api::BootInfo;
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator as X86FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

use super::frame::Frame;
use super::memory_map::usable_regions;

/// Tracks up to 1 GiB of 4 KiB frames (262144 frames, 32 KiB bitmap in .bss).
const MAX_FRAMES: usize = 262144;
const BITMAP_WORDS: usize = MAX_FRAMES / 64;

/// Zero-initialized in .bss — never touched on the boot stack.
static FRAME_BITMAP: Mutex<[u64; BITMAP_WORDS]> = Mutex::new([0; BITMAP_WORDS]);

#[derive(Debug, Clone, Copy)]
struct FrameMeta {
    base_frame: usize,
    frame_count: usize,
    used_count: usize,
    initialized: bool,
}

static FRAME_META: Mutex<FrameMeta> = Mutex::new(FrameMeta {
    base_frame: 0,
    frame_count: 0,
    used_count: 0,
    initialized: false,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    OutOfRange,
    DoubleFree,
    NotInitialized,
    SelfTestFailed,
    Exhausted,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameStats {
    pub total: usize,
    pub used: usize,
    pub free: usize,
}

fn is_free(bitmap: &[u64; BITMAP_WORDS], idx: usize) -> bool {
    (bitmap[idx / 64] >> (idx % 64)) & 1 == 0
}

fn set_used(bitmap: &mut [u64; BITMAP_WORDS], idx: usize, used: bool) {
    let bit = 1u64 << (idx % 64);
    if used {
        bitmap[idx / 64] |= bit;
    } else {
        bitmap[idx / 64] &= !bit;
    }
}

fn frame_index(meta: &FrameMeta, frame: Frame) -> Option<usize> {
    frame.number.checked_sub(meta.base_frame)
}

fn allocate_frame_inner(meta: &mut FrameMeta, bitmap: &mut [u64; BITMAP_WORDS]) -> Option<Frame> {
    for idx in 0..meta.frame_count {
        if is_free(bitmap, idx) {
            set_used(bitmap, idx, true);
            meta.used_count += 1;
            return Some(Frame {
                number: meta.base_frame + idx,
            });
        }
    }
    None
}

fn deallocate_frame_inner(
    meta: &mut FrameMeta,
    bitmap: &mut [u64; BITMAP_WORDS],
    frame: Frame,
) -> Result<(), FrameError> {
    let idx = frame_index(meta, frame).ok_or(FrameError::OutOfRange)?;
    if idx >= meta.frame_count {
        return Err(FrameError::OutOfRange);
    }
    if is_free(bitmap, idx) {
        return Err(FrameError::DoubleFree);
    }
    set_used(bitmap, idx, false);
    meta.used_count -= 1;
    Ok(())
}

pub struct BootFrameAllocator;

impl BootFrameAllocator {
    fn with_inner<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&mut FrameMeta, &mut [u64; BITMAP_WORDS]) -> R,
    {
        let mut meta = FRAME_META.lock();
        if !meta.initialized {
            return None;
        }
        let mut bitmap = FRAME_BITMAP.lock();
        Some(f(&mut meta, &mut bitmap))
    }
}

unsafe impl X86FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        BootFrameAllocator::with_inner(|meta, bitmap| {
            allocate_frame_inner(meta, bitmap).map(|frame| {
                PhysFrame::from_start_address(PhysAddr::new(frame.start_address() as u64))
                    .expect("allocated frame must be page-aligned")
            })
        })
        .flatten()
    }
}

pub fn init_frame_allocator(boot_info: &BootInfo) -> BootFrameAllocator {
    let (start, end) = usable_regions(&boot_info.memory_regions)
        .max_by_key(|(start, end)| end.saturating_sub(*start))
        .expect("no usable memory");

    let base_frame = start / super::frame::PAGE_SIZE;
    let end_frame = end / super::frame::PAGE_SIZE;
    let frame_count = end_frame.saturating_sub(base_frame);

    assert!(
        frame_count <= MAX_FRAMES,
        "usable RAM region too large for bitmap ({} frames, max {})",
        frame_count,
        MAX_FRAMES
    );

    let mut meta = FRAME_META.lock();
    let mut bitmap = FRAME_BITMAP.lock();

    meta.base_frame = base_frame;
    meta.frame_count = frame_count;
    meta.used_count = 0;
    meta.initialized = true;
    bitmap.fill(0);

    BootFrameAllocator
}

pub fn stats() -> Result<FrameStats, FrameError> {
    let meta = FRAME_META.lock();
    if !meta.initialized {
        return Err(FrameError::NotInitialized);
    }
    Ok(FrameStats {
        total: meta.frame_count,
        used: meta.used_count,
        free: meta.frame_count.saturating_sub(meta.used_count),
    })
}

pub fn allocate_frame() -> Result<Frame, FrameError> {
    BootFrameAllocator::with_inner(allocate_frame_inner)
        .flatten()
        .ok_or(FrameError::Exhausted)
}

pub fn deallocate_frame(frame: Frame) -> Result<(), FrameError> {
    BootFrameAllocator::with_inner(|meta, bitmap| deallocate_frame_inner(meta, bitmap, frame))
        .ok_or(FrameError::NotInitialized)?
}

/// Allocate/deallocate cycle; returns frames recovered to the free pool.
pub fn self_test(rounds: usize) -> Result<usize, FrameError> {
    const MAX_ROUNDS: usize = 64;
    let rounds = rounds.min(MAX_ROUNDS);

    let before = stats()?.used;
    let mut frames = [None; MAX_ROUNDS];

    for slot in frames.iter_mut().take(rounds) {
        *slot = Some(allocate_frame()?);
    }

    for slot in frames.iter().take(rounds) {
        deallocate_frame(slot.unwrap())?;
    }

    let after = stats()?.used;
    if after != before {
        return Err(FrameError::SelfTestFailed);
    }

    Ok(rounds)
}
