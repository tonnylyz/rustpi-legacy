use cortex_a::{asm::*, regs::*};

use crate::arch::aarch64::core::CORES;
use crate::board::BOARD_CORE_NUMBER;
use crate::lib::page_table::PageTableTrait;
use crate::lib::process::Process;
use crate::lib::scheduler::SchedulerTrait;
use crate::mm::PageFrame;

pub type Arch = Aarch64Arch;

pub type ContextFrame = super::context_frame::Aarch64ContextFrame;

pub type PageTable = super::page_table::Aarch64PageTable;

pub type ArchPageTableEntry = super::page_table::Aarch64PageTableEntry;

pub type AddressSpaceId = u16;

pub struct Aarch64Arch;

impl crate::arch::ArchTrait for Aarch64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn start_first_process() -> ! {
    use cortex_a::regs::*;
    use core::intrinsics::size_of;
    extern {
      fn pop_context() -> !;
    }
    unsafe {
      let ctx = SP.get() as usize - size_of::<ContextFrame>();
      let context = ctx as *mut ContextFrame;
      *context = (*crate::arch::Arch::running_process().unwrap().pcb()).context.unwrap();
      SP.set(ctx as u64);
      pop_context();
    }
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

  fn core_id() -> usize {
    MPIDR_EL1.get() as usize & (BOARD_CORE_NUMBER - 1)
  }

  fn context() -> *mut ContextFrame {
    unsafe {
      let ctx = CORES[Self::core_id()].context;
      if ctx == 0 {
        panic!("arch: interface: context null");
      }
      ctx as *mut ContextFrame
    }
  }

  fn has_context() -> bool {
    unsafe {
      let ctx = CORES[Self::core_id()].context;
      ctx != 0
    }
  }

  fn running_process() -> Option<Process> {
    unsafe {
      CORES[Self::core_id()].running_process
    }
  }

  fn set_running_process(p: Option<Process>) {
    unsafe {
      CORES[Self::core_id()].running_process = p;
    }
  }

  fn schedule() {
    unsafe {
      CORES[Self::core_id()].scheduler.schedule();
    }
  }
}
