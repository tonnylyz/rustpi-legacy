// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use cortex_a::{barrier, regs::*};

global_asm!(include_str!("exception.S"));

#[repr(C)]
struct TrapFrame {
  spsr: u64,
  elr: u64,
  esr: u64,
  sp: u64,
  gpr: [u64; 31],
}

#[no_mangle]
static mut TRAP_FRAME : TrapFrame = TrapFrame {
  spsr: 0,
  elr: 0,
  esr: 0,
  sp: 0,
  gpr: [0; 31],
};

//--------------------------------------------------------------------------------------------------
// Current, EL0
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_el0_synchronous() {
  loop {}
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq() {
  println!("current_el0_irq");
  loop {}
}

#[no_mangle]
unsafe extern "C" fn current_el0_serror() {
  loop {}
}

//--------------------------------------------------------------------------------------------------
// Current, ELx
//--------------------------------------------------------------------------------------------------

/// Asynchronous exception taken from the current EL, using SP of the current EL.
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous() {
  loop {}
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
  loop {}
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch64
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous() {
  loop {}
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq() {
  println!("lower_aarch64_irq");
  loop {}
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror() {
  loop {}
}

pub unsafe fn set_vbar_el1() {
  extern "C" {
    static mut vectors: u64;
  }
  let addr: u64 = &vectors as *const _ as u64;
  VBAR_EL1.set(addr);
  barrier::isb(barrier::SY);
}
