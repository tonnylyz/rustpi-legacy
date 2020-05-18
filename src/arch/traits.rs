use crate::{
  arch::ContextFrame,
};
use crate::lib::thread::Thread;

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
  fn context(&self) -> &ContextFrame;
  fn context_mut(&self) -> &mut ContextFrame;
  fn set_context(&self, ctx: *mut ContextFrame);
  fn clear_context(&self);
  fn has_context(&self) -> bool;
  fn running_thread(&self) -> Option<Thread>;
  fn set_running_thread(&self, p: Option<Thread>);
  fn schedule(&self);
  fn create_idle_thread(&self);
  fn idle_thread(&self) -> Thread;
}

pub trait ContextFrameTrait {
  fn new(pc: usize, sp: usize, arg: usize, privileged: bool) -> Self;

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