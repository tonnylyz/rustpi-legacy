// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use cortex_a::{barrier, regs::*};

use crate::arch::{ContextFrame, CoreTrait};

global_asm!(include_str!("exception.S"));

//--------------------------------------------------------------------------------------------------
// Current, EL0
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_el0_synchronous() {
  panic!("current_el0_synchronous");
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq(ctx: *mut ContextFrame) {
  lower_aarch64_irq(ctx);
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
unsafe extern "C" fn lower_aarch64_synchronous(ctx: *mut ContextFrame) {
  use crate::lib::isr::*;
  let core = crate::lib::core::current();
  core.set_context(ctx);
  if ESR_EL1.matches_all(ESR_EL1::EC::SVC64) {
    Isr::system_call();
  } else if ESR_EL1.matches_all(ESR_EL1::EC::InstrAbortLowerEL) | ESR_EL1.matches_all(ESR_EL1::EC::DataAbortLowerEL) {
    Isr::page_fault();
  } else {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    println!("lower_aarch64_synchronous: ec {:06b}", ec);
    Isr::default();
  }
  core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(ctx: *mut ContextFrame) {
  use crate::lib::isr::*;
  let core = crate::lib::core::current();
  core.set_context(ctx);
  Isr::interrupt_request();
  core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(ctx: *mut ContextFrame) {
  use crate::lib::isr::*;
  let core = crate::lib::core::current();
  core.set_context(ctx);
  Isr::default();
  core.clear_context();
}

pub fn init() {
  extern "C" {
    fn vectors();
  }
  unsafe {
    let addr: u64 = vectors as usize as u64;
    VBAR_EL1.set(addr);
    barrier::isb(barrier::SY);
  }
}
