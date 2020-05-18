use core::ops::Range;

use crate::arch::{Address, ArchTrait, CoreTrait};
use crate::lib::current_core;
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};

pub const BOARD_CORE_NUMBER: usize = 4;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0x4000_0000;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x3f00_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x3f00_0000..0x4000_0000;

pub fn init() {
  crate::driver::uart::init();
}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::arch::Arch::exception_init();

  current_core().create_idle_thread();
}

pub fn launch_other_cores() {
  let page_table = crate::arch::PageTable::kernel_page_table();
  page_table.map(0x4000_0000, 0x4000_0000, EntryAttribute::kernel_device());

  unsafe {
    extern "C" {
      fn get_el2_entry() -> u64;
    }
    let entry = get_el2_entry();
    (0xe0usize.pa2kva() as *mut u64).write(entry);
    (0xe8usize.pa2kva() as *mut u64).write(entry);
    (0xf0usize.pa2kva() as *mut u64).write(entry);
  }
}
