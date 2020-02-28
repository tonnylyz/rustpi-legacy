use core::ops::Range;

pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0x4000_0000;

pub struct Rpi3;

impl super::Board for Rpi3 {
  #[inline(always)]
  fn physical_address_limit(&self) -> usize {
    BOARD_PHYSICAL_ADDRESS_LIMIT
  }

  #[inline(always)]
  fn normal_memory_range(&self) -> Range<usize> {
    0x0000_0000..0x3f00_0000
  }

  #[inline(always)]
  fn device_memory_range(&self) -> Range<usize> {
    0x3f00_0000..0x4000_0000
  }

  #[inline(always)]
  fn kernel_start(&self) -> usize {
    0x0008_0000
  }

  #[inline(always)]
  fn kernel_stack_top(&self) -> usize {
    0x0008_0000
  }
}

pub static BOARD: Rpi3 = Rpi3;