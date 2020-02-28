// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;
use core::ops::Range;
use arch::pa2kva;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(range: Range<usize>) {
  unsafe {
    HEAP_ALLOCATOR.lock().init(pa2kva(range.start), range.end - range.start)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("Kernel heap exhausted");
}