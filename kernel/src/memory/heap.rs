use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(heap_start: usize, heap_size: usize) {
    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start as *mut u8, heap_size);
    }
}

pub struct HeapStats {
    pub used: usize,
    pub free: usize,
    pub total: usize,
}

pub fn stats() -> HeapStats {
    let heap = ALLOCATOR.lock();
    HeapStats {
        used: heap.used(),
        free: heap.free(),
        total: heap.size(),
    }
}
