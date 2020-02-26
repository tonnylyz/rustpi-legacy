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
    TrapFrame{
      gpr: [0; 31],
      spsr: 0,
      elr: 0x80000,
      sp: 0x8000_0000,
    }
  }
}

#[no_mangle]
pub static mut TRAP_FRAME : TrapFrame = TrapFrame {
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
  println!("lower_aarch64_synchronous elr {:016x} x0 {:016x}", ELR_EL1.get(), TRAP_FRAME.gpr[0]);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq() {
  panic!("lower_aarch64_irq");
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror() {
  panic!("lower_aarch64_serror");
}

pub unsafe fn set_vbar_el1() {
  extern "C" {
    static mut vectors: u64;
  }
  let addr: u64 = &vectors as *const _ as u64;
  VBAR_EL1.set(addr);
  barrier::isb(barrier::SY);
}
