use cortex_a::{asm::*, regs::*};

use lib::page_table::PageTableTrait;

use crate::mm::PageFrame;

pub type Arch = Aarch64Arch;

pub type ContextFrame = super::context_frame::Aarch64ContextFrame;

pub type PageTable = super::page_table::Aarch64PageTable;

pub type ArchPageTableEntry = super::page_table::Aarch64PageTableEntry;

pub type AddressSpaceId = u16;

#[no_mangle]
pub static mut CONTEXT_FRAME: ContextFrame = super::context_frame::Aarch64ContextFrame::zero();

pub struct Aarch64Arch;

impl crate::arch::ArchTrait for Aarch64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn start_first_process() -> ! {
    extern {
      fn pop_time_stack() -> !;
    }
    unsafe { pop_time_stack(); }
  }

  fn kernel_page_table() -> PageTable {
    let frame = PageFrame::new(cortex_a::regs::TTBR1_EL1.get_baddr() as usize);
    PageTable::new(frame)
  }

  fn user_page_table() -> PageTable {
    let frame = PageFrame::new(cortex_a::regs::TTBR0_EL1.get_baddr() as usize);
    PageTable::new(frame)
  }

  fn set_user_page_table(pt: PageTable, asid: AddressSpaceId) {
    use cortex_a::{regs::*, *};
    unsafe {
      TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64));
      TTBR0_EL1.set_baddr(pt.directory().pa() as u64);
      barrier::isb(barrier::SY);
      barrier::dsb(barrier::SY);
    }
  }

  fn invalidate_tlb() {
    unsafe {
      asm!("dsb ishst");
      asm!("tlbi vmalle1is");
      asm!("dsb ish");
      asm!("isb");
    }
  }

  fn wait_for_event() {
    wfe();
  }

  fn nop() {
    nop();
  }

  fn fault_address() -> usize {
    FAR_EL1.get() as usize
  }
}
