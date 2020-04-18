use core::ops::Range;

use crate::arch::{Address, PAGE_SIZE};
use crate::lib::round_up;

// non paged memory in kernel (kernel heap memory)
pub const CONFIG_NON_PAGED_MEMORY_SIZE: usize = 0xf00_0000;

pub fn paged_range() -> Range<usize> {
  extern "C" {
    // Note: link-time label, see kernel.aarch64.lds
    fn KERNEL_END();
  }
  let kernel_end = round_up((KERNEL_END as usize).kva2pa(), PAGE_SIZE);
  let normal_range = crate::board::BOARD_NORMAL_MEMORY_RANGE;
  kernel_end..(normal_range.end - CONFIG_NON_PAGED_MEMORY_SIZE)
}

pub fn heap_range() -> Range<usize> {
  let normal_range = crate::board::BOARD_NORMAL_MEMORY_RANGE;
  (normal_range.end - CONFIG_NON_PAGED_MEMORY_SIZE)..normal_range.end
}
