// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use cortex_a::{barrier, regs::*};
use core::fmt::{Formatter, Error};

global_asm!(include_str!("exception.S"));

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
    Aarch64ContextFrame {
      gpr: [0; 31],
      spsr: (SPSR_EL1::M::EL0t + SPSR_EL1::I::Unmasked).value as u64,
      elr: 0xdeadbeef,
      sp: 0xdeadbeef,
    }
  }
}

impl crate::arch::traits::ContextFrameImpl for Aarch64ContextFrame {
  fn get_syscall_argument(&self, i: usize) -> usize {
    const AARCH64_SYSCALL_ARG_LIMIT: usize = 8;
    assert!(i < AARCH64_SYSCALL_ARG_LIMIT);
    // x0 ~ x7
    self.gpr[i] as usize
  }

  fn get_syscall_number(&self) -> usize {
    // x8
    self.gpr[8] as usize
  }

  fn set_syscall_return_value(&mut self, v: usize) {
    // x0
    self.gpr[0] = v as u64;
  }

  fn get_exception_pc(&self) -> usize {
    self.elr as usize
  }

  fn set_exception_pc(&mut self, pc: usize) {
    self.elr = pc as u64;
  }

  fn get_stack_pointer(&self) -> usize {
    self.sp as usize
  }

  fn set_stack_pointer(&mut self, sp: usize) {
    self.sp = sp as u64;
  }

  fn set_argument(&mut self, arg: usize) {
    self.gpr[0] = arg as u64;
  }
}

//--------------------------------------------------------------------------------------------------
// Current, EL0
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_el0_synchronous() {
  panic!("current_el0_synchronous");
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq() {
  panic!("current_el0_irq");
}

#[no_mangle]
unsafe extern "C" fn current_el0_serror() {
  panic!("current_el0_serror");
}

//--------------------------------------------------------------------------------------------------
// Current, ELx
//--------------------------------------------------------------------------------------------------

/// Asynchronous exception taken from the current EL, using SP of the current EL.
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous() {
  panic!("current_elx_synchronous {:016x}", cortex_a::regs::FAR_EL1.get());
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq() {
  panic!("current_elx_irq");
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror() {
  panic!("current_elx_serror");
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch64
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous() {
  use crate::lib::isr::*;
  if ESR_EL1.matches_all(ESR_EL1::EC::SVC64) {
    ISR.system_call();
  } else if ESR_EL1.matches_all(ESR_EL1::EC::InstrAbortLowerEL) | ESR_EL1.matches_all(ESR_EL1::EC::DataAbortLowerEL) {
    ISR.page_fault();
  } else {
    //let ec = ESR_EL1.read(ESR_EL1::EC);
    //panic!("lower_aarch64_synchronous ec {:06b}", ec);
    ISR.default();
  }
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq() {
  use crate::lib::isr::*;
  ISR.interrupt_request();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror() {
  use crate::lib::isr::*;
  ISR.default();
}

pub fn init() {
  extern "C" {
    static mut vectors: u64;
  }
  unsafe {
    let addr: u64 = &vectors as *const _ as u64;
    VBAR_EL1.set(addr);
    barrier::isb(barrier::SY);
  }
}
