// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn allocator_init() {
  unsafe {
    HEAP_ALLOCATOR.lock().init(0xFFFFFF8000000000 + 0x3000_0000, 0x0f00_0000)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  println!("alloc_error_handler");
  loop {}
}