// export types and functions

use arch::traits::{Arch,PageTableImpl};
use mm::PageFrame;

pub type PageTable = super::page_table::Aarch64PageTable;

pub type ContextFrame = super::exception::Aarch64ContextFrame;

#[no_mangle]
pub static mut CONTEXT_FRAME: ContextFrame = ContextFrame {
  gpr: [0; 31],
  spsr: 0,
  elr: 0,
  sp: 0,
};

pub struct Aarch64Arch;

impl Arch for Aarch64Arch {
  fn exception_init(&self) {
    super::exception::init();
  }

  fn start_first_process(&self) -> !{
    extern {
      fn pop_time_stack() -> !;
    }
    unsafe { pop_time_stack(); }
  }

  fn get_kernel_page_table(&self) -> PageTable {
    let directory = super::mmu::kernel_page_table_directory_pa();
    let frame = PageFrame::new(directory);
    PageTable::new(frame)
  }
}

pub static ARCH: Aarch64Arch = Aarch64Arch;