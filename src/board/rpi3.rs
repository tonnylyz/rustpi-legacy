use core::ops::Range;

pub const BOARD_CORE_NUMBER: usize = 4;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0x4000_0000;

pub struct Rpi3;

impl super::BoardTrait for Rpi3 {
  #[inline(always)]
  fn physical_address_limit() -> usize {
    BOARD_PHYSICAL_ADDRESS_LIMIT
  }

  #[inline(always)]
  fn normal_memory_range() -> Range<usize> {
    0x0000_0000..0x3f00_0000
  }

  #[inline(always)]
  fn device_memory_range() -> Range<usize> {
    0x3f00_0000..0x4000_0000
  }

  #[inline(always)]
  fn kernel_start() -> usize {
    0x0008_0000
  }

  #[inline(always)]
  fn kernel_stack_top() -> usize {
    0x0008_0000
  }

  #[inline(always)]
  fn kernel_stack_top_core(core_id: usize) -> usize {
    Self::kernel_stack_top() - core_id * 0x0002_0000
  }
}

pub type Board = Rpi3;