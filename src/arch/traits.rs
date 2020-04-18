use crate::{
  arch::ContextFrame,
  lib::process::Process,
};

pub trait Address {
  fn pa2kva(&self) -> usize;
  fn kva2pa(&self) -> usize;
}

pub trait ArchTrait {
  fn exception_init();

  // Note: kernel runs at privileged mode
  // need to trigger a half process switching
  // Require: a process has been schedule, its
  // context filled in CONTEXT_FRAME, and its
  // page table installed at low address space.
  fn invalidate_tlb();
  fn wait_for_event();
  fn nop();
  fn fault_address() -> usize;
  fn core_id() -> usize;
}

pub trait CoreTrait {
  fn current() -> *mut Self;
  fn context(&self) -> Option<*mut ContextFrame>;
  fn set_context(&mut self, ctx: Option<*mut ContextFrame>);
  fn install_context(&self, ctx: ContextFrame);
  fn running_process(&self) -> Option<Process>;
  fn set_running_process(&mut self, p: Option<Process>);
  fn schedule(&mut self);
  fn start_first_process(&self) -> !;
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
  fn from_pte(value: usize) -> Self;
  fn from_pa(pa: usize) -> Self;
  fn to_pte(&self) -> usize;
  fn to_pa(&self) -> usize;
  fn to_kva(&self) -> usize;
  fn valid(&self) -> bool;
  fn entry(&self, index: usize) -> Self;
  fn set_entry(&self, index: usize, value: Self);
  fn alloc_table() -> Self;
}