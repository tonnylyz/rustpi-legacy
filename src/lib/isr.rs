use cortex_a::regs::{RegisterReadWrite};
use arch::{CONTEXT_FRAME, ContextFrameImpl};
use lib::process::CURRENT_PROCESS;
use config::{CONFIG_USER_LIMIT};
use mm::PageFrame;
use core::intrinsics::size_of;
use lib::process::ProcessStatus::{PsRunnable, PsNotRunnable, PsFree};

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
          let ps = match arg(1) {
            1 => { PsRunnable },
            2 => { PsNotRunnable },
            _ => { PsFree }
          };
          match SystemCall::process_set_status(arg(0) as u16, ps) {
            Ok(_) => {r = None},
            Err(e) => {r = Some(e as usize)},
          }
        }
        11 => {
          SystemCall::ipc_receive(arg(0));
        }
        12 => {
          match SystemCall::ipc_can_send(arg(0) as u16, arg(1), arg(2), arg(3)) {
            Ok(_) => {r = Some(0)},
            Err(e) => {r = Some(e as usize)},
          }
        }
        _ => { println!("system call: unrecognized system call number"); }
      }
      if let Some(value) = r {
        CONTEXT_FRAME.set_syscall_return_value(value);
      }
    }
  }
  fn interrupt_request(&self) {
    //println!("InterruptServiceRoutine: interrupt_request");
    //println!("{}", unsafe { CONTEXT_FRAME });
    crate::driver::timer::next(0);
    crate::lib::scheduler::schedule();
  }
  fn page_fault(&self) {
    use arch::*;
    unsafe {
      let addr = cortex_a::regs::FAR_EL1.get() as usize;
      let va = round_down(addr, PAGE_SIZE);
      if addr < CONFIG_USER_LIMIT {
        let p = CURRENT_PROCESS.unwrap();
        if (*p.pcb()).exception_handler != 0 {
          let page_table = (*p.pcb()).directory.unwrap();
          if let Some(pte) = page_table.lookup_page(va) {
            if pte.attr.copy_on_write {
              assert!(size_of::<ContextFrame>() < PAGE_SIZE);
              let stack_top = (*p.pcb()).exception_stack_top;
              let stack_btm = stack_top - PAGE_SIZE;
              if let Some(stack_pte) = page_table.lookup_page(stack_btm) {
                let stack_frame = PageFrame::new(stack_pte.addr);
                core::intrinsics::volatile_copy_memory(
                  (stack_frame.kva() + PAGE_SIZE - size_of::<ContextFrame>()) as *mut ContextFrame,
                  (&CONTEXT_FRAME) as *const ContextFrame,
                  1
                );
                let mut ctx = CONTEXT_FRAME.clone();
                ctx.set_exception_pc((*p.pcb()).exception_handler);
                ctx.set_stack_pointer(stack_top - size_of::<ContextFrame>());
                ctx.set_argument(va);
                CONTEXT_FRAME = ctx;
                return;
              }
            }
          }
        }
      }
    }
    //
    //let addr = cortex_a::regs::FAR_EL1.get() as usize;
    //let va = round_down(addr, PAGE_SIZE);
    //if addr < CONFIG_MEM_USER_LIMIT {
    //  unsafe {
    //    let page_table = (*CURRENT_PROCESS.unwrap().pcb()).directory.unwrap();
    //    if let Some(pte) = page_table.lookup_page(va) {
    //      if pte.attr.copy_on_write {
    //        let frame = PageFrame::new(pte.addr);
    //        let new_frame = crate::mm::page_pool::alloc();
    //        new_frame.copy_from(&frame);
    //        page_table.remove_page(va);
    //        page_table.insert_page(va, new_frame, pte.attr - PageTableEntryAttr::copy_on_write() + PageTableEntryAttr::writable());
    //        println!("copy on write: {:08x} ok!", va);
    //        return;
    //      }
    //    }
    //  }
    //}
    unsafe {
      println!("\nInterruptServiceRoutine: page_fault");
      println!("far: {:016x}", cortex_a::regs::FAR_EL1.get());
      println!("{:?}", (*(CURRENT_PROCESS.unwrap().pcb())).directory.unwrap().lookup_page(cortex_a::regs::FAR_EL1.get() as usize));
      println!("CONTEXT_FRAME:\n{}", CONTEXT_FRAME);
      println!("{}", *CURRENT_PROCESS.unwrap().pcb());
      panic!("InterruptServiceRoutine: page_fault");
    }
  }
  fn default(&self) {
    panic!("InterruptServiceRoutine: default");
  }
}

// ARCH code use crate::lib::isr::ISR to invoke kernel function
pub static ISR: Isr = Isr;