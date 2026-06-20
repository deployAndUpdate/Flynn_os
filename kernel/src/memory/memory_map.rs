use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use core::fmt::Write;

use crate::logger::Logger;

pub fn print_memory_map(memory_regions: &MemoryRegions) {
    let mut logger = Logger;

    writeln!(
        logger,
        "[boot] memory map received ({} regions)",
        memory_regions.len()
    )
    .ok();

    let mut usable_bytes: u64 = 0;

    for region in memory_regions.iter() {
        let size = region.end.saturating_sub(region.start);
        let kind = region_kind_label(region.kind);

        if is_usable(region.kind) {
            usable_bytes = usable_bytes.saturating_add(size);
        }

        writeln!(
            logger,
            "  {kind}: {:#018x}..{:#018x} ({size} bytes)",
            region.start, region.end,
        )
        .ok();
    }

    writeln!(logger, "[boot] usable RAM: {usable_bytes} bytes").ok();
}

fn region_kind_label(kind: MemoryRegionKind) -> &'static str {
    match kind {
        MemoryRegionKind::Usable => "usable",
        MemoryRegionKind::Bootloader => "bootloader",
        MemoryRegionKind::UnknownUefi(_) => "unknown-uefi",
        MemoryRegionKind::UnknownBios(_) => "unknown-bios",
        _ => "unknown",
    }
}

/// Returns true if the region can be freely used by the kernel allocator.
pub fn is_usable(kind: MemoryRegionKind) -> bool {
    kind == MemoryRegionKind::Usable
}

pub fn usable_regions(memory_regions: &MemoryRegions) -> impl Iterator<Item = (usize, usize)> + '_ {
    memory_regions.iter().filter_map(|region| {
        if is_usable(region.kind) {
            Some((region.start as usize, region.end as usize))
        } else {
            None
        }
    })
}
