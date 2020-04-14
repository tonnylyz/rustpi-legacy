use core::fmt::Formatter;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64ContextFrame {
  // TODO: fill context
}

impl core::fmt::Display for Riscv64ContextFrame {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    unimplemented!()
  }
}

impl Default for Riscv64ContextFrame {
  fn default() -> Self {
    unimplemented!()
  }
}

impl crate::arch::traits::ContextFrameTrait for Riscv64ContextFrame {
  fn syscall_argument(&self, i: usize) -> usize {
    unimplemented!()
  }

  fn syscall_number(&self) -> usize {
    unimplemented!()
  }

  fn set_syscall_return_value(&mut self, v: usize) {
    unimplemented!()
  }

  fn exception_pc(&self) -> usize {
    unimplemented!()
  }

  fn set_exception_pc(&mut self, pc: usize) {
    unimplemented!()
  }

  fn stack_pointer(&self) -> usize {
    unimplemented!()
  }

  fn set_stack_pointer(&mut self, sp: usize) {
    unimplemented!()
  }

  fn set_argument(&mut self, arg: usize) {
    unimplemented!()
  }
}

