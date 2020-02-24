// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Exception handling.

use cortex_a::{asm, barrier, regs::*};
use register::InMemoryRegister;

// Assembly counterpart to this file.
global_asm!(include_str!("exception.S"));

/// Wrapper struct for memory copy of SPSR_EL1.
#[repr(transparent)]
struct SpsrEL1(InMemoryRegister<u32, SPSR_EL1::Register>);

/// The exception context as it is stored on the stack on exception entry.
#[repr(C)]
struct ExceptionContext {
  // General Purpose Registers.
  gpr: [u64; 30],
  // The link register, aka x30.
  lr: u64,
  // Exception link register. The program counter at the time the exception happened.
  elr_el1: u64,
  // Saved program status.
  spsr_el1: SpsrEL1,
}

/// Wrapper struct for pretty printing ESR_EL1.
struct EsrEL1;

//--------------------------------------------------------------------------------------------------
// Exception vector implementation
//--------------------------------------------------------------------------------------------------

/// Print verbose information about the exception and the panic.
fn default_exception_handler(e: &ExceptionContext) {
  println!("default_exception_handler");
  loop {}
}

//--------------------------------------------------------------------------------------------------
// Current, EL0
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Current, ELx
//--------------------------------------------------------------------------------------------------

/// Asynchronous exception taken from the current EL, using SP of the current EL.
#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
  let far_el1 = FAR_EL1.get();

  // This catches the demo case for this tutorial. If the fault address happens to be 8 GiB,
  // advance the exception link register for one instruction, so that execution can continue.
  if far_el1 == 8 * 1024 * 1024 * 1024 {
    e.elr_el1 += 4;

    asm::eret()
  }

  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch64
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Lower, AArch32
//--------------------------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
  default_exception_handler(e);
}

//--------------------------------------------------------------------------------------------------
// Arch-public
//--------------------------------------------------------------------------------------------------

/// Set the exception vector base address register.
///
/// # Safety
///
/// - The vector table and the symbol `__exception_vector_table_start` from the linker script must
///   adhere to the alignment and size constraints demanded by the AArch64 spec.
pub unsafe fn set_vbar_el1() {
  // Provided by exception.S.
  extern "C" {
    static mut __exception_vector_start: u64;
  }
  let addr: u64 = &__exception_vector_start as *const _ as u64;

  VBAR_EL1.set(addr);

  // Force VBAR update to complete before next instruction.
  barrier::isb(barrier::SY);
}

pub trait DaifField {
  fn daif_field() -> register::Field<u32, DAIF::Register>;
}

pub struct Debug;
pub struct SError;
pub struct IRQ;
pub struct FIQ;

impl DaifField for Debug {
  fn daif_field() -> register::Field<u32, DAIF::Register> {
    DAIF::D
  }
}

impl DaifField for SError {
  fn daif_field() -> register::Field<u32, DAIF::Register> {
    DAIF::A
  }
}

impl DaifField for IRQ {
  fn daif_field() -> register::Field<u32, DAIF::Register> {
    DAIF::I
  }
}

impl DaifField for FIQ {
  fn daif_field() -> register::Field<u32, DAIF::Register> {
    DAIF::F
  }
}

pub fn is_masked<T: DaifField>() -> bool {
  DAIF.is_set(T::daif_field())
}
