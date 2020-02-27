// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use cortex_a::{barrier, regs::*};

global_asm!(include_str!("exception.S"));

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrapFrame {
  pub gpr: [u64; 31],
  pub spsr: u64,
  pub elr: u64,
  pub sp: u64,
}

impl TrapFrame {
  pub fn default() -> Self {
    TrapFrame {
      gpr: [0; 31],
      spsr: (SPSR_EL1::M::EL0t + SPSR_EL1::I::Unmasked).value as u64,
      elr: 0x80000,
      sp: 0x8000_0000,
    }
  }
}

//#[repr(transparent)]
//struct SpsrEl1(LocalRegisterCopy<u32, SPSR_EL1::Register>);
//
//impl core::convert::From<u64> for SpsrEl1 {
//  fn from(r: u64) -> Self {
//    SpsrEl1(LocalRegisterCopy::new(r as u32))
//  }
//}

#[no_mangle]
pub static mut TRAP_FRAME: TrapFrame = TrapFrame {
  gpr: [0; 31],
  spsr: 0,
  elr: 0,
  sp: 0,
};

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
  panic!("current_elx_synchronous");
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq() {
  println!("current_elx_irq");
  println!("elr {:016x}", TRAP_FRAME.elr);
  println!("sp {:016x}", TRAP_FRAME.sp);
  loop {}
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
  if ESR_EL1.matches_all(ESR_EL1::EC::SVC64) {
    // system call (treat as an print i)
    println!("{}", TRAP_FRAME.gpr[0]);
  } else if ESR_EL1.matches_all(ESR_EL1::EC::InstrAbortLowerEL) | ESR_EL1.matches_all(ESR_EL1::EC::DataAbortLowerEL) {
    // maybe page fault
    panic!("EL0 abort")
  } else {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    println!("lower_aarch64_synchronous ec {:06b}", ec);
  }
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq() {
  println!("lower_aarch64_irq");
  crate::driver::timer::timer_next(0);
  super::process::process_schedule();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror() {
  panic!("lower_aarch64_serror");
}

pub fn exception_init() {
  extern "C" {
    static mut vectors: u64;
  }
  unsafe {
    let addr: u64 = &vectors as *const _ as u64;
    VBAR_EL1.set(addr);
    barrier::isb(barrier::SY);
  }
}
