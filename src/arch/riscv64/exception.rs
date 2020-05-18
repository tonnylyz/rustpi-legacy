use riscv::regs::*;

use crate::arch::{ContextFrame, ContextFrameTrait, CoreTrait};
use crate::lib::isr::{InterruptServiceRoutine, Isr};

global_asm!(include_str!("exception.S"));

#[derive(Debug)]
enum Interrupt {
  UserSoftware = 0,
  SupervisorSoftware = 1,
  MachineSoftware = 3,
  UserTimer = 4,
  SupervisorTimer = 5,
  MachineTimer = 7,
  UserExternal = 8,
  SupervisorExternal = 9,
  MachineExternal = 11,
  Unknown,
}

impl core::convert::From<usize> for Interrupt {
  fn from(u: usize) -> Self {
    match u {
      0 => Interrupt::UserSoftware,
      1 => Interrupt::SupervisorSoftware,
      3 => Interrupt::MachineSoftware,
      4 => Interrupt::UserTimer,
      5 => Interrupt::SupervisorTimer,
      7 => Interrupt::MachineTimer,
      8 => Interrupt::UserExternal,
      9 => Interrupt::SupervisorExternal,
      11 => Interrupt::MachineExternal,
      _ => Interrupt::Unknown
    }
  }
}

#[derive(Debug)]
enum Exception {
  InstructionAddressMisaligned = 0,
  InstructionAccessFault = 1,
  IllegalInstruction = 2,
  Breakpoint = 3,
  LoadAddressMisaligned = 4,
  LoadAccessFault = 5,
  StoreAddressMisaligned = 6,
  StoreAccessFault = 7,
  EnvironmentCallFromUserMode = 8,
  EnvironmentCallFromSupervisorMode = 9,
  EnvironmentCallFromMachineMode = 11,
  InstructionPageFault = 12,
  LoadPageFault = 13,
  StorePageFault = 15,
  Unknown,
}

impl core::convert::From<usize> for Exception {
  fn from(u: usize) -> Self {
    match u {
      0 => Exception::InstructionAddressMisaligned,
      1 => Exception::InstructionAccessFault,
      2 => Exception::IllegalInstruction,
      3 => Exception::Breakpoint,
      4 => Exception::LoadAddressMisaligned,
      5 => Exception::LoadAccessFault,
      6 => Exception::StoreAddressMisaligned,
      7 => Exception::StoreAccessFault,
      8 => Exception::EnvironmentCallFromUserMode,
      9 => Exception::EnvironmentCallFromSupervisorMode,
      11 => Exception::EnvironmentCallFromMachineMode,
      12 => Exception::InstructionPageFault,
      13 => Exception::LoadPageFault,
      15 => Exception::StorePageFault,
      _ => Exception::Unknown
    }
  }
}

#[no_mangle]
unsafe extern "C" fn exception_entry(ctx: usize) {
  let from_kernel = SSTATUS.is_set(SSTATUS::SPP);
  if from_kernel {
    //panic!("kernel exception");
  }
  let core = crate::lib::core::current();
  core.set_context(ctx as *mut ContextFrame);
  let cause = SCAUSE.get();
  let irq = (cause >> 63) != 0;
  let code = (cause & 0xf) as usize;
  if irq {
    match Interrupt::from(code) {
      Interrupt::UserSoftware => { panic!("Interrupt::UserSoft") }
      Interrupt::SupervisorSoftware => { panic!("Interrupt::SupervisorSoft") }
      Interrupt::UserTimer => { panic!("Interrupt::UserTimer") }
      Interrupt::SupervisorTimer => {
        Isr::interrupt_request()
      }
      Interrupt::UserExternal => { panic!("Interrupt::UserExternal") }
      Interrupt::SupervisorExternal => { panic!("Interrupt::SupervisorExternal") }
      _ => { panic!("Interrupt::Unknown") }
    }
  } else {
    match Exception::from(code) {
      Exception::InstructionAddressMisaligned => { panic!("Exception::InstructionMisaligned") }
      Exception::InstructionAccessFault => { panic!("Exception::InstructionFault") }
      Exception::IllegalInstruction => { Isr::default() }
      Exception::Breakpoint => { panic!("Exception::Breakpoint") }
      Exception::LoadAccessFault => { panic!("Exception::LoadFault") }
      Exception::StoreAddressMisaligned => { panic!("Exception::StoreMisaligned") }
      Exception::StoreAccessFault => {
        println!("{:016x}", STVAL.get());
        panic!("Exception::StoreFault")
      }
      Exception::EnvironmentCallFromUserMode => {
        // Note: we need to set epc to next instruction before doing system call
        //       pay attention to yield and process_alloc
        let pc = core.context_mut().exception_pc();
        core.context_mut().set_exception_pc(pc + 4);
        Isr::system_call();
      }
      Exception::InstructionPageFault => { Isr::page_fault() }
      Exception::LoadPageFault => { Isr::page_fault() }
      Exception::StorePageFault => { Isr::page_fault() }
      _ => { panic!("Exception::Unknown") }
    }
  }
  core.clear_context();
}

pub fn init() {
  extern "C" {
    fn push_context();
  }
  STVEC.write(STVEC::BASE.val(push_context as usize as u64 >> 2) + STVEC::MODE::Direct);
  // Note: riscv vector only 4 byte per cause
  //       direct mode make it distributed later in `exception_entry`
}
