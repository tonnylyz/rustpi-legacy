use core::ops::Range;
use crate::arch::ArchTrait;

pub const BOARD_CORE_NUMBER: usize = 1;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0xc000_0000;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xc000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub fn init() {
  crate::driver::uart::init();
  crate::driver::plic::init();
}

pub fn init_per_core() {
  let core_id = crate::arch::Arch::core_id();
  crate::driver::timer::init(core_id);
}