// export types and functions

use arch::traits::{Arch,PageTableImpl};
use mm::PageFrame;

pub type PageTable = super::page_table::Aarch64PageTable;

pub type ContextFrame = super::exception::Aarch64ContextFrame;

pub type AddressSpaceId = u16;

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
    let frame = PageFrame::new(cortex_a::regs::TTBR1_EL1.get_baddr() as usize);
    PageTable::new(frame)
  }

  fn get_user_page_table(&self) -> PageTable {
    let frame = PageFrame::new(cortex_a::regs::TTBR0_EL1.get_baddr() as usize);
    PageTable::new(frame)
  }

  fn set_user_page_table(&self, pt: PageTable, asid: AddressSpaceId) {
    use cortex_a::{regs::*, *};
    unsafe {
      TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64));
      TTBR0_EL1.set_baddr(pt.directory().pa() as u64);
      barrier::isb(barrier::SY);
      barrier::dsb(barrier::SY);
    }
  }
}

pub static ARCH: Aarch64Arch = Aarch64Arch;