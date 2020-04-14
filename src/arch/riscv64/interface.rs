use crate::lib::process::Process;

pub type Arch = Riscv64Arch;

pub type ContextFrame = super::context_frame::Riscv64ContextFrame;

pub type PageTable = super::page_table::Riscv64PageTable;

pub type ArchPageTableEntry = super::page_table::Riscv64PageTableEntry;

// TODO: placeholder here
pub type AddressSpaceId = u16;

pub struct Riscv64Arch;

impl crate::arch::ArchTrait for Riscv64Arch {
  fn exception_init() {
    unimplemented!()
  }

  fn start_first_process() -> ! {
    unimplemented!()
  }

  fn kernel_page_table() -> PageTable {
    unimplemented!()
  }

  fn user_page_table() -> PageTable {
    unimplemented!()
  }

  fn set_user_page_table(pt: PageTable, asid: AddressSpaceId) {
    unimplemented!()
  }

  fn invalidate_tlb() {
    unimplemented!()
  }

  fn wait_for_event() {
    unimplemented!()
  }

  fn nop() {
    unimplemented!()
  }

  fn fault_address() -> usize {
    unimplemented!()
  }

  fn core_id() -> usize {
    unimplemented!()
  }

  fn context() -> *mut ContextFrame {
    unimplemented!()
  }

  fn has_context() -> bool {
    unimplemented!()
  }

  fn running_process() -> Option<Process> {
    unimplemented!()
  }

  fn set_running_process(p: Option<Process>) {
    unimplemented!()
  }

  fn schedule() {
    unimplemented!()
  }
}