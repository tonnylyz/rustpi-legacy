// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;
use core::ops::Range;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn allocator_init(range : Range<usize>) {
  unsafe {
    HEAP_ALLOCATOR.lock().init(0xFFFFFF8000000000 + range.start, range.end - range.start)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("Kernel heap exhausted");
}