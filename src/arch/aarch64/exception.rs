// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use cortex_a::{barrier, regs::*};

global_asm!(include_str!("exception.S"));

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Aarch64ContextFrame {
  pub gpr: [u64; 31],
  pub spsr: u64,
  pub elr: u64,
  pub sp: u64,
}

impl crate::arch::traits::ContextFrameImpl for Aarch64ContextFrame {
  fn default() -> Self {
    Aarch64ContextFrame {
      gpr: [0; 31],
      spsr: (SPSR_EL1::M::EL0t + SPSR_EL1::I::Unmasked).value as u64,
      elr: 0x80000,
      sp: 0x8000_0000,
    }
  }

  fn system_call_argument(&self, i: usize) -> usize {
    const AARCH64_SYSCALL_ARG_LIMIT: usize = 8;
    if i > AARCH64_SYSCALL_ARG_LIMIT {
      panic!("Fetch argument index exceeds limit {}/{}", i, AARCH64_SYSCALL_ARG_LIMIT);
    }
    // x0 ~ x7
    self.gpr[i] as usize
  }

  fn system_call_number(&self) -> usize {
    // x8
    self.gpr[8] as usize
  }

  fn system_call_set_return_value(&mut self, v: usize) {
    // x0
    self.gpr[0] = v as u64;
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
