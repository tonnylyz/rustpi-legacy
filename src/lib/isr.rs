use core::intrinsics::size_of;

use crate::arch::*;
use crate::config::*;
use crate::lib::page_table::PageTableTrait;
use crate::lib::process::Pid;
use crate::mm::PageFrame;

pub trait InterruptServiceRoutine {
  fn system_call();
  fn interrupt_request();
  fn page_fault();
  fn default();
}

pub struct Isr;

#[derive(Debug)]
pub enum SystemCallResult {
  Void,
  Pid(Pid),
  R(Option<isize>),
}

pub trait SystemCallResultOk {
  fn to_isize(&self) -> isize;
}

impl SystemCallResultOk for () {
  fn to_isize(&self) -> isize {
    0
  }
}

impl SystemCallResultOk for u16 {
  fn to_isize(&self) -> isize {
    self.clone() as isize
  }
}

impl core::convert::From<Pid> for SystemCallResult {
  fn from(pid: Pid) -> Self {
    SystemCallResult::Pid(pid)
  }
}

impl core::convert::From<()> for SystemCallResult {
  fn from(_: ()) -> Self {
    SystemCallResult::Void
  }
}

impl<T> core::convert::From<Result<T, super::syscall::Error>> for SystemCallResult where T: SystemCallResultOk {
  fn from(sce: Result<T, super::syscall::Error>) -> Self {
    SystemCallResult::R(
      match sce {
        Ok(t) => { Some(t.to_isize()) }
        Err(e) => { Some(-(e as isize)) }
      }
    )
  }
}

impl InterruptServiceRoutine for Isr {
  fn system_call() {
    use crate::lib::syscall::*;
    unsafe {
      let arg = |i: usize| { (*crate::arch::Arch::context()).syscall_argument(i) };
      let scr = match (*crate::arch::Arch::context()).syscall_number() {
        1 => {
          //print!("core_{}: putc({})", crate::arch::Arch::core_id(), arg(0) as u8 as char);
          //println!();
          //().into()
          SystemCall::putc(arg(0) as u8 as char).into()
        }
        2 => {
          SystemCall::getpid().into()
        }
        3 => {
          SystemCall::process_yield().into()
        }
        4 => {
          SystemCall::process_destroy(arg(0) as u16).into()
        }
        5 => {
          SystemCall::process_set_exception_handler(arg(0) as u16, arg(1), arg(2)).into()
        }
        6 => {
          SystemCall::mem_alloc(arg(0) as u16, arg(1), arg(2)).into()
        }
        7 => {
          SystemCall::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)).into()
        }
        8 => {
          SystemCall::mem_unmap(arg(0) as u16, arg(1)).into()
        }
        9 => {
          SystemCall::process_alloc().into()
        }
        10 => {
          use crate::lib::process::Status::{PsRunnable, PsNotRunnable, PsFree};
          let ps = match arg(1) {
            1 => { PsRunnable }
            2 => { PsNotRunnable }
            _ => { PsFree }
          };
          SystemCall::process_set_status(arg(0) as u16, ps).into()
        }
        11 => {
          SystemCall::ipc_receive(arg(0)).into()
        }
        12 => {
          SystemCall::ipc_can_send(arg(0) as u16, arg(1), arg(2), arg(3)).into()
        }
        _ => { println!("system call: unrecognized system call number").into() }
      };
      match scr {
        SystemCallResult::Void => {}
        SystemCallResult::Pid(pid) => {
          //println!("[{}]:{}:{:?}", CURRENT.unwrap().pid(), (*crate::arch::Arch::context()).syscall_number(), scr);
          (*crate::arch::Arch::context()).set_syscall_return_value(pid as usize);
        }
        SystemCallResult::R(o) => {
          //println!("[{}]:{}:{:?}", CURRENT.unwrap().pid(), (*crate::arch::Arch::context()).syscall_number(), scr);
          match o {
            None => { (*crate::arch::Arch::context()).set_syscall_return_value(0); }
            Some(i) => { (*crate::arch::Arch::context()).set_syscall_return_value(i as usize); }
          }
        }
      }
    }
  }
  fn interrupt_request() {
    let core_id = crate::arch::Arch::core_id();
    //println!("core_{} irq", core_id);
    //println!("{:016x}", crate::arch::Arch::context() as usize);
    unsafe {
      let irq_source = crate::driver::mmio::read_word(pa2kva(0x4000_0060 + 4 * core_id));
      if (irq_source >> 1) & 0b1 == 0 {
        panic!("core_{} irq source not timer", core_id);
      }
    }
    crate::driver::timer::next(0);
    crate::lib::scheduler::schedule();
    //println!("core_{} irq return", core_id);
  }
  fn page_fault() {
    unsafe {
      let addr = Arch::fault_address();
      let va = round_down(addr, PAGE_SIZE);
      if va >= CONFIG_USER_LIMIT {
        println!("isr: page_fault: va >= CONFIG_USER_LIMIT, process killed");
        crate::arch::Arch::running_process().unwrap().destroy();
        return;
      }
      let p = crate::arch::Arch::running_process().unwrap();
      if (*p.pcb()).exception_handler == 0 {
        println!("isr: page_fault: process has no handler, process killed");
        crate::arch::Arch::running_process().unwrap().destroy();
        return;
      }
      let page_table = (*p.pcb()).page_table.unwrap();
      let stack_top = (*p.pcb()).exception_stack_top;
      let stack_btm = stack_top - PAGE_SIZE;
      match page_table.lookup_page(stack_btm) {
        Some(stack_pte) => {
          if va == stack_btm {
            println!("isr: page_fault: fault on exception stack, process killed");
            crate::arch::Arch::running_process().unwrap().destroy();
            return;
          }
          let stack_frame = PageFrame::new(stack_pte.pa());
          core::intrinsics::volatile_copy_memory(
            (stack_frame.kva() + PAGE_SIZE - size_of::<ContextFrame>()) as *mut ContextFrame,
            crate::arch::Arch::context() as *const ContextFrame,
            1,
          );
          let mut ctx = (*crate::arch::Arch::context()).clone();
          ctx.set_exception_pc((*p.pcb()).exception_handler);
          ctx.set_stack_pointer(stack_top - size_of::<ContextFrame>());
          ctx.set_argument(va);
          (*crate::arch::Arch::context()) = ctx;
          return;
        }
        None => {
          println!("isr: page_fault: exception stack not valid, process killed");
          crate::arch::Arch::running_process().unwrap().destroy();
          return;
        }
      }
    }
  }
  fn default() {
    unsafe {
      println!("fault_address: {:016x}", crate::Arch::fault_address());
      println!("{:?}", (*crate::arch::Arch::running_process().unwrap().pcb()).page_table.unwrap().lookup_page(crate::Arch::fault_address()));
      println!("pc: {:016x}", (*crate::arch::Arch::context()).exception_pc());
      println!("{:?}", (*crate::arch::Arch::running_process().unwrap().pcb()).page_table.unwrap().lookup_page((*crate::arch::Arch::context()).exception_pc()));
      println!("{}", (*crate::arch::Arch::context()));
      println!("isr: default: process killed");
      crate::arch::Arch::running_process().unwrap().destroy();
    }
  }
}
