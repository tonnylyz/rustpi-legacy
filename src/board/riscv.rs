use core::ops::Range;

use crate::arch::{ArchTrait, CoreTrait};
use crate::lib::current_core;

#[allow(dead_code)]
pub const BOARD_CORE_NUMBER: usize = 1;
#[allow(dead_code)]
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0xc000_0000;
#[allow(dead_code)]
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xc000_0000;
#[allow(dead_code)]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub fn init() {
  crate::driver::uart::init();
  crate::driver::plic::init();
}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::arch::Arch::exception_init();
  current_core().create_idle_thread();
}

pub fn launch_other_cores() {}