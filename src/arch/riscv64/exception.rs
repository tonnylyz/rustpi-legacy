use riscv::register::*;
use crate::arch::ContextFrame;
use riscv::register::scause::{Trap, Interrupt, Exception};
use crate::lib::isr::{Isr, InterruptServiceRoutine};

global_asm!(include_str!("exception.S"));

#[no_mangle]
unsafe extern "C" fn exception_entry(ctx: usize) {
  super::interface::CONTEXT = Some(ctx);
  let cause = scause::read();
  println!("{:?}", cause.cause());
  match cause.cause() {
    Trap::Interrupt(i) => {
      match i {
        Interrupt::UserSoft => {Isr::default()},
        Interrupt::SupervisorSoft => {Isr::default()},
        Interrupt::UserTimer => {Isr::default()},
        Interrupt::SupervisorTimer => {Isr::interrupt_request()},
        Interrupt::UserExternal => {Isr::default()},
        Interrupt::SupervisorExternal => {Isr::default()},
        Interrupt::Unknown => {Isr::default()},
      }
    },
    Trap::Exception(e) => {
      match e {
        Exception::InstructionMisaligned => {Isr::default()},
        Exception::InstructionFault => {Isr::default()},
        Exception::IllegalInstruction => {Isr::default()},
        Exception::Breakpoint => {Isr::default()},
        Exception::LoadFault => {Isr::default()},
        Exception::StoreMisaligned => {Isr::default()},
        Exception::StoreFault => {Isr::default()},
        Exception::UserEnvCall => {Isr::system_call()},
        Exception::InstructionPageFault => {Isr::default()},
        Exception::LoadPageFault => {Isr::default()},
        Exception::StorePageFault => {Isr::default()},
        Exception::Unknown => {Isr::default()},
      }
    },
  }
  super::interface::CONTEXT = None;
}

pub fn init() {
  extern "C" {
    fn push_context();
  }
  unsafe {
    sscratch::write(0);
    stvec::write(push_context as usize, stvec::TrapMode::Direct);
    // Note: riscv vector only 4 byte per cause
    //       direct mode make it distributed later in `exception_entry`
  }
}
