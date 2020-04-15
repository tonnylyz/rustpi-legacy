use core::ops::Range;

pub const BOARD_CORE_NUMBER: usize = 1;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0x4000_0000;

pub struct Riscv;

impl super::BoardTrait for Riscv {
  fn physical_address_limit() -> usize {
    BOARD_PHYSICAL_ADDRESS_LIMIT
  }

  fn normal_memory_range() -> Range<usize> {
    unimplemented!()
  }

  fn device_memory_range() -> Range<usize> {
    unimplemented!()
  }

  fn kernel_start() -> usize {
    unimplemented!()
  }

  fn kernel_stack_top() -> usize {
    unimplemented!()
  }

  fn kernel_stack_top_core(core_id: usize) -> usize {
    unimplemented!()
  }
}

pub type Board = Riscv;