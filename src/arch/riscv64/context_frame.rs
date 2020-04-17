use core::fmt::Formatter;
use riscv::regs::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64ContextFrame {
  gpr: [u64; 32],
  sstatus: u64,
  sepc: u64,
}

impl core::fmt::Display for Riscv64ContextFrame {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    for i in 0..32 {
      write!(f, "x{:02}: {:016x}   ", i, self.gpr[i])?;
      if (i + 1) % 2 == 0 {
        write!(f, "\n")?;
      }
    }
    write!(f, "sst: {:016x}", self.sstatus)?;
    writeln!(f, "   epc: {:016x}", self.sepc)?;
    Ok(())
  }
}

impl Default for Riscv64ContextFrame {
  fn default() -> Self {
    unsafe {
      let mut sstatus = SSTATUS.get();
      // Note: The SIE bit enables or disables all interrupts in supervisor mode. When SIE is clear, interrupts
      // are not taken while in supervisor mode. When the hart is running in user-mode, the value in
      // SIE is ignored, and supervisor-level interrupts are enabled. The supervisor can disable individual
      // interrupt sources using the sie CSR.
      // Note: The SPIE bit indicates whether supervisor interrupts were enabled prior to trapping into supervisor
      // mode. When a trap is taken into supervisor mode, SPIE is set to SIE, and SIE is set to 0. When
      // an SRET instruction is executed, SIE is set to SPIE, then SPIE is set to 1.
      Riscv64ContextFrame {
        gpr: [0xdeadbeef_deadbeef; 32],
        sstatus: (SSTATUS::SD.val(1) + SSTATUS::FS.val(0b11) + SSTATUS::SPP::User + SSTATUS::SPIE.val(1) + SSTATUS::SIE.val(0)).value,
        sepc: 0xdeadbeef_deadbeef,
      }
    }
  }
}

impl crate::arch::traits::ContextFrameTrait for Riscv64ContextFrame {
  fn syscall_argument(&self, i: usize) -> usize {
    assert!(i <= 5);
    // a0 ~ a5 -> x10 ~ x15
    self.gpr[i + 10] as usize
  }

  fn syscall_number(&self) -> usize {
    // a7 -> x17
    self.gpr[17] as usize
  }

  fn set_syscall_return_value(&mut self, v: usize) {
    // a0 -> x10
    self.gpr[10] = v as u64;
  }

  fn exception_pc(&self) -> usize {
    self.sepc as usize
  }

  fn set_exception_pc(&mut self, pc: usize) {
    self.sepc = pc as u64;
  }

  fn stack_pointer(&self) -> usize {
    // sp -> x2
    self.gpr[2] as usize
  }

  fn set_stack_pointer(&mut self, sp: usize) {
    self.gpr[2] = sp as u64;
  }

  fn set_argument(&mut self, arg: usize) {
    self.gpr[10] = arg as u64;
  }
}

