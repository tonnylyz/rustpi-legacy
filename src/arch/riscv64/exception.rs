use riscv::register::*;
use crate::arch::{ContextFrame, ArchTrait, ContextFrameTrait};
use riscv::register::scause::{Trap, Interrupt, Exception};
use crate::lib::isr::{Isr, InterruptServiceRoutine};

global_asm!(include_str!("exception.S"));

#[no_mangle]
unsafe extern "C" fn exception_entry(ctx: usize) {
  super::interface::CONTEXT = Some(ctx);
  let cause = scause::read();
  //println!("{:?}", cause.cause());
  match cause.cause() {
    Trap::Interrupt(i) => {
      match i {
        Interrupt::UserSoft => {panic!("Interrupt::UserSoft")},
        Interrupt::SupervisorSoft => {panic!("Interrupt::SupervisorSoft")},
        Interrupt::UserTimer => {panic!("Interrupt::UserTimer")},
        Interrupt::SupervisorTimer => {Isr::interrupt_request()},
        Interrupt::UserExternal => {panic!("Interrupt::UserExternal")},
        Interrupt::SupervisorExternal => {panic!("Interrupt::SupervisorExternal")},
        Interrupt::Unknown => {panic!("Interrupt::Unknown")},
      }
    },
    Trap::Exception(e) => {
      match e {
        Exception::InstructionMisaligned => {panic!("Exception::InstructionMisaligned")},
        Exception::InstructionFault => {panic!("Exception::InstructionFault")},
        Exception::IllegalInstruction => {panic!("Exception::IllegalInstruction")},
        Exception::Breakpoint => {panic!("Exception::Breakpoint")},
        Exception::LoadFault => {panic!("Exception::LoadFault")},
        Exception::StoreMisaligned => {panic!("Exception::StoreMisaligned")},
        Exception::StoreFault => {

          println!("{:016x}", stval::read());
          panic!("Exception::StoreFault")
        },
        Exception::UserEnvCall => {
          Isr::system_call();
          let pc = (*super::interface::Arch::context()).exception_pc();
          (*super::interface::Arch::context()).set_exception_pc(pc + 4);
        },
        Exception::InstructionPageFault => {Isr::page_fault()},
        Exception::LoadPageFault => {Isr::page_fault()},
        Exception::StorePageFault => {Isr::page_fault()},
        Exception::Unknown => {panic!("Exception::Unknown")},
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
