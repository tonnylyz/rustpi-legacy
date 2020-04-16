use riscv::register::*;

use crate::arch::{PAGE_SHIFT, PAGE_SIZE};
use crate::lib::page_table::PageTableTrait;
use crate::lib::process::Process;
use crate::lib::scheduler::{RoundRobinScheduler, SchedulerTrait};
use crate::mm::PageFrame;

pub type Arch = Riscv64Arch;

pub type ContextFrame = super::context_frame::Riscv64ContextFrame;

pub type PageTable = super::page_table::Riscv64PageTable;

pub type ArchPageTableEntry = super::page_table::Riscv64PageTableEntry;

pub type AddressSpaceId = u16;

pub struct Riscv64Arch;

pub static mut CONTEXT: Option<usize> = None;
static mut PROCESS: Option<Process> = None;
static mut SCHEDULER: RoundRobinScheduler = RoundRobinScheduler::new();

impl crate::arch::ArchTrait for Riscv64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn start_first_process() -> ! {
    use core::intrinsics::size_of;
    extern {
      fn pop_context_first(ctx: usize) -> !;
    }
    unsafe {
      let context = (*crate::arch::Arch::running_process().unwrap().pcb()).context.unwrap();
      //sie::set_stimer();
      pop_context_first(&context as *const ContextFrame as usize)
    }
  }

  fn kernel_page_table() -> PageTable {
    PageTable::new(PageFrame::new(satp::read().ppn() << PAGE_SHIFT))
  }

  fn user_page_table() -> PageTable {
    // Note user and kernel share page directory
    PageTable::new(PageFrame::new(satp::read().ppn() << PAGE_SHIFT))
  }

  fn set_user_page_table(pt: PageTable, asid: AddressSpaceId) {
    unsafe {
      let directory = pt.directory().kva() as *mut [usize; 512];
      // TODO: fix this hard coded mapping
      (*directory)[508] = ((0x00000000 >> 12) << 10) | 0xcf;
      (*directory)[509] = ((0x40000000 >> 12) << 10) | 0xcf;
      (*directory)[510] = ((0x80000000 >> 12) << 10) | 0xcf;
      (*directory)[511] = ((0xc0000000 >> 12) << 10) | 0xcf;
      //for i in 0..512 {
      //  if (*directory)[i] != 0 {
      //    println!("{}:{:016x}", i, (*directory)[i]);
      //  }
      //}
      satp::set(satp::Mode::Sv39, 0/*asid as usize*/, pt.directory().pa() >> PAGE_SHIFT);
      riscv::asm::sfence_vma_all();
    }
  }

  fn invalidate_tlb() {
    unsafe {
      riscv::asm::sfence_vma_all();
    }
  }

  fn wait_for_event() {
    unsafe {
      riscv::asm::wfi();
    }
  }

  fn nop() {
    unsafe {
      asm!("nop");
    }
  }

  fn fault_address() -> usize {
    stval::read()
  }

  fn core_id() -> usize {
    0
  }

  fn context() -> *mut ContextFrame {
    unsafe {
      CONTEXT.unwrap() as *mut ContextFrame
    }
  }

  fn has_context() -> bool {
    unsafe {
      CONTEXT.is_some()
    }
  }

  fn running_process() -> Option<Process> {
    unsafe {
      PROCESS
    }
  }

  fn set_running_process(p: Option<Process>) {
    unsafe {
      PROCESS = p;
    }
  }

  fn schedule() {
    unsafe {
      SCHEDULER.schedule();
    }
  }
}