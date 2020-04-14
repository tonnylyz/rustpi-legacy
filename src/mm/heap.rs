use core::ops::Range;

// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;

use crate::arch::pa2kva;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(range: Range<usize>) {
  unsafe {
    HEAP_ALLOCATOR.lock().init(pa2kva(range.start), range.end - range.start)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}