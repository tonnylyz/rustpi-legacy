use cortex_a::regs::RegisterReadWrite;
use lib::scheduler::Scheduler;
use arch::{CONTEXT_FRAME, ContextFrameImpl};
use lib::process::{ProcessStatus, CURRENT_PROCESS};

pub trait InterruptServiceRoutine {
  fn system_call(&self);
  fn interrupt_request(&self);
  fn page_fault(&self);
  fn default(&self);
}

pub struct Isr;

impl InterruptServiceRoutine for Isr {
  fn system_call(&self) {
    use lib::syscall::*;
    unsafe {
      let mut r: Option<usize> = None;
      let arg = |i: usize| { CONTEXT_FRAME.get_syscall_argument(i) };
      match CONTEXT_FRAME.get_syscall_number() {
        1 => SystemCall::putc(arg(0) as u8 as char),
        2 => {
          r = Some(SystemCall::getpid() as usize);
        }
        3 => SystemCall::process_yield(),
        4 => {
          match SystemCall::process_destroy(arg(0) as u16) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        5 => {
          match SystemCall::process_set_exception_handler(arg(0) as u16, arg(1), arg(2)) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        6 => {
          match SystemCall::mem_alloc(arg(0) as u16, arg(1), arg(2)) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        7 => {
          match SystemCall::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        8 => {
          match SystemCall::mem_unmap(arg(0) as u16, arg(1)) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        9 => {
          match SystemCall::process_alloc() {
            Ok(pid) => {r = Some(pid as usize) },
            Err(e) => {r = Some(usize::max_value() - e as usize) },
          }
        }
        10 => {
          match SystemCall::process_set_status(arg(0) as u16, match arg(1) {
            1 => ProcessStatus::PsRunnable,
            2 => ProcessStatus::PsNotRunnable,
            _ => ProcessStatus::PsNotRunnable, // TODO: handle this invalid argument
          }) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        11 => {
          SystemCall::ipc_receive(arg(0));
        }
        12 => {
          match SystemCall::ipc_can_send(arg(0) as u16, arg(1), arg(2), arg(3)) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        _ => { println!("Unrecognized system call number"); println!("{:?}", CONTEXT_FRAME); }
      }
      if let Some(value) = r {
        CONTEXT_FRAME.set_syscall_return_value(value);
      }
    }
  }
  fn interrupt_request(&self) {
    //println!("InterruptServiceRoutine: interrupt_request");
    crate::driver::timer::timer_next(0);
    unsafe {
      super::scheduler::SCHEDULER.schedule(super::process_pool::pid_list());
    }
  }
  fn page_fault(&self) {
    use arch::*;
    println!("elr: {:016x}", unsafe { CONTEXT_FRAME.get_exception_pc() });
    println!("far: {:016x}", cortex_a::regs::FAR_EL1.get());
    println!("{}", unsafe { CONTEXT_FRAME });
    println!("{:?}", unsafe { *CURRENT_PROCESS.unwrap().pcb() });
    panic!("InterruptServiceRoutine: page_fault");
  }
  fn default(&self) {
    panic!("InterruptServiceRoutine: default");
  }
}

// ARCH code use crate::lib::isr::ISR to invoke kernel function
pub static ISR: Isr = Isr;