use core::fmt::Formatter;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64ContextFrame {
  gpr: [u64; 31],
  spsr: u64,
  elr: u64,
  sp: u64,
}

impl core::fmt::Display for Aarch64ContextFrame {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    for i in 0..31 {
      write!(f, "x{:02}: {:016x}   ", i, self.gpr[i])?;
      if (i + 1) % 2 == 0 {
        write!(f, "\n")?;
      }
    }
    writeln!(f, "spsr:{:016x}", self.spsr)?;
    write!(f, "elr: {:016x}", self.elr)?;
    writeln!(f, "   sp:  {:016x}", self.sp)?;
    Ok(())
  }
}

impl Aarch64ContextFrame {
  pub const fn zero() -> Self {
    Aarch64ContextFrame {
      gpr: [0; 31],
      spsr: 0,
      elr: 0,
      sp: 0,
    }
  }
}

impl Default for Aarch64ContextFrame {
  fn default() -> Self {
    use cortex_a::regs::*;
    Aarch64ContextFrame {
      gpr: [0; 31],
      spsr: (SPSR_EL1::M::EL0t + SPSR_EL1::I::Unmasked).value as u64,
      elr: 0xdeadbeef,
      sp: 0xdeadbeef,
    }
  }
}

impl crate::arch::traits::ContextFrameTrait for Aarch64ContextFrame {
  fn syscall_argument(&self, i: usize) -> usize {
    const AARCH64_SYSCALL_ARG_LIMIT: usize = 8;
    assert!(i < AARCH64_SYSCALL_ARG_LIMIT);
    // x0 ~ x7
    self.gpr[i] as usize
  }

  fn syscall_number(&self) -> usize {
    // x8
    self.gpr[8] as usize
  }

  fn set_syscall_return_value(&mut self, v: usize) {
    // x0
    self.gpr[0] = v as u64;
  }

  fn exception_pc(&self) -> usize {
    self.elr as usize
  }

  fn set_exception_pc(&mut self, pc: usize) {
    self.elr = pc as u64;
  }

  fn stack_pointer(&self) -> usize {
    self.sp as usize
  }

  fn set_stack_pointer(&mut self, sp: usize) {
    self.sp = sp as u64;
  }

  fn set_argument(&mut self, arg: usize) {
    self.gpr[0] = arg as u64;
  }
}
