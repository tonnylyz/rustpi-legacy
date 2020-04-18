// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;

use crate::arch::Address;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
  let range = super::config::heap_range();
  unsafe {
    HEAP_ALLOCATOR.lock().init(range.start.pa2kva(), range.end - range.start)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}