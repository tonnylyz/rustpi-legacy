use core::ops::Range;

pub const BOARD_CORE_NUMBER: usize = 1;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0xc000_0000;

pub struct Riscv;

impl super::BoardTrait for Riscv {
  fn physical_address_limit() -> usize {
    BOARD_PHYSICAL_ADDRESS_LIMIT
  }

  fn normal_memory_range() -> Range<usize> {
    0x8000_0000..0xc000_0000
  }

  fn device_memory_range() -> Range<usize> {
    0..0x8000_0000
  }

  fn kernel_start() -> usize {
    0x8020_0000
  }

  fn kernel_stack_top() -> usize {
    extern {
      fn BOOT_STACK_TOP();
    }
    unsafe {
      BOOT_STACK_TOP as usize
    }
  }

  fn kernel_stack_top_core(core_id: usize) -> usize {
    // TODO: differentiate core stack
    extern {
      fn BOOT_STACK_TOP();
    }
    unsafe {
      BOOT_STACK_TOP as usize
    }
  }
}

pub type Board = Riscv;