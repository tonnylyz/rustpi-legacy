use core::intrinsics::size_of;

use crate::arch::*;
use crate::config::*;
use crate::lib::{current_core, current_process, current_thread, round_down};
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
    let ctx = current_core().context_mut();
    let arg = |i: usize| { ctx.syscall_argument(i) };
    let scr = match ctx.syscall_number() {
      1 => {
        //print!("core_{}: putc({})", crate::arch::Arch::core_id(), arg(0) as u8 as char);
        //println!();
        //().into()
        SystemCall::putc(arg(0) as u8 as char).into()
      }
      2 => {
        SystemCall::get_pid().into()
      }
      3 => {
        SystemCall::thread_yield().into()
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
        use crate::lib::thread::Status::*;
        match arg(1) {
          1 => { SystemCall::thread_set_status(arg(0) as u16, TsRunnable).into() }
          2 => { SystemCall::thread_set_status(arg(0) as u16, TsNotRunnable).into() }
          _ => { ().into() }
        }
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
        //println!("{}:{:?}", (*ctx).syscall_number(), scr);
        ctx.set_syscall_return_value(pid as usize);
      }
      SystemCallResult::R(o) => {
        //println!("{}:{:?}", (*ctx).syscall_number(), scr);
        match o {
          None => { ctx.set_syscall_return_value(0); }
          Some(i) => { ctx.set_syscall_return_value(i as usize); }
        }
      }
    }
  }

  fn interrupt_request() {
    crate::driver::timer::next();
    crate::lib::scheduler::schedule();
  }

  fn page_fault() {
    let t = current_thread();
    if t.is_none() {
      panic!("isr: page_fault: no running thread");
    }
    let p = current_process();
    if p.is_none() {
      panic!("isr: page_fault: no running process");
    }
    let p = p.unwrap();

    let addr = Arch::fault_address();
    let va = round_down(addr, PAGE_SIZE);
    if va >= CONFIG_USER_LIMIT {
      println!("isr: page_fault: va >= CONFIG_USER_LIMIT, process killed");
      p.destroy();
      return;
    }
    if p.exception_handler().is_none() {
      println!("isr: page_fault: process has no handler, process killed");
      p.destroy();
      return;
    }
    let (entry, stack_top) = p.exception_handler().unwrap();
    let page_table = p.page_table();
    let stack_btm = stack_top - PAGE_SIZE;
    match page_table.lookup_page(stack_btm) {
      Some(stack_pte) => {
        if va == stack_btm {
          println!("isr: page_fault: fault on exception stack, process killed");
          p.destroy();
          return;
        }
        let ctx = current_core().context_mut();

        let stack_frame = PageFrame::new(stack_pte.pa());
        unsafe {
          core::intrinsics::volatile_copy_memory(
            (stack_frame.kva() + PAGE_SIZE - size_of::<ContextFrame>()) as *mut ContextFrame,
            ctx as *mut ContextFrame,
            1,
          );
          ctx.set_exception_pc(entry);
          ctx.set_stack_pointer(stack_top - size_of::<ContextFrame>());
          ctx.set_argument(va);
        }
        return;
      }
      None => {
        println!("isr: page_fault: exception stack not valid, process killed");
        p.destroy();
        return;
      }
    }
  }

  fn default() {
    match current_thread() {
      None => { panic!("isr: default: no running thread") }
      Some(t) => {
        match t.process() {
          None => { panic!("isr: default: no running process") }
          Some(p) => {
            println!("isr: default: process killed");
            p.destroy();
          }
        }
      }
    }
  }
}
