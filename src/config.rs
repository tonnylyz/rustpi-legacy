// non paged memory in kernel (kernel heap memory)
pub const CONFIG_NON_PAGED_MEMORY_SIZE: usize = 0xf00_0000;

use core::ops::Range;
use arch::*;
use board::*;

// helper functions
pub fn paged_range() -> Range<usize> {
  extern "C" {
    // Note: link-time label, see kernel.lds
    fn KERNEL_END();
  }
  let mut kernel_end = kva2pa(KERNEL_END as usize);
  kernel_end = (kernel_end >> PAGE_SHIFT) << PAGE_SHIFT;
  kernel_end += PAGE_SIZE; // align to page
  let normal_range = BOARD.normal_memory_range();
  kernel_end..(normal_range.end - CONFIG_NON_PAGED_MEMORY_SIZE)
}

pub fn heap_range() -> Range<usize> {
  let normal_range = BOARD.normal_memory_range();
  (normal_range.end - CONFIG_NON_PAGED_MEMORY_SIZE)..normal_range.end
}

// Process
pub const CONFIG_PROCESS_NUMBER: usize = 1 << 10;
pub const CONFIG_PROCESS_STACK_TOP: usize = 0x8000_0000;
pub const CONFIG_MEM_USER_LIMIT: usize = 0xc000_0000;
