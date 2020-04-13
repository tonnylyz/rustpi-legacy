use arch::{AddressSpaceId, PageTable, ContextFrame};
use lib::process::Process;

pub trait ArchTrait {
  fn exception_init();

  // Note: kernel runs at privileged mode
  // need to trigger a half process switching
  // Require: a process has been schedule, its
  // context filled in CONTEXT_FRAME, and its
  // page table installed at low address space.
  fn start_first_process() -> !;

  fn kernel_page_table() -> PageTable;
  fn user_page_table() -> PageTable;
  fn set_user_page_table(pt: PageTable, asid: AddressSpaceId);

  fn invalidate_tlb();
  fn wait_for_event();
  fn nop();
  fn fault_address() -> usize;
  fn core_id() -> usize;
  fn context() -> *mut ContextFrame;
  fn has_context() -> bool;
  fn running_process() -> Option<Process>;
  fn set_running_process(p: Option<Process>);
  fn schedule();
}

pub trait ContextFrameTrait: Default {
  fn syscall_argument(&self, i: usize) -> usize;
  fn syscall_number(&self) -> usize;
  fn set_syscall_return_value(&mut self, v: usize);
  fn exception_pc(&self) -> usize;
  fn set_exception_pc(&mut self, pc: usize);
  fn stack_pointer(&self) -> usize;
  fn set_stack_pointer(&mut self, sp: usize);
  fn set_argument(&mut self, arg: usize);
}

pub trait ArchPageTableEntryTrait {
  fn new(value: usize) -> Self;
  fn to_usize(&self) -> usize;
}