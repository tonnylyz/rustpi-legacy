use crate::arch::{ArchPageTableEntry, ArchPageTableEntryTrait, ContextFrameTrait, CoreTrait, PAGE_SIZE};
use crate::config::CONFIG_USER_LIMIT;
use crate::lib::{current_process, current_thread, round_down};
use crate::lib::page_table::{Entry, PageTableEntryAttrTrait, PageTableTrait};
use crate::lib::process::{Pid, Process};

use self::Error::*;

pub enum Error {
  InvalidArgumentError = 1,
  _OutOfProcessError,
  OutOfMemoryError,
  ProcessPidNotFoundError,
  ProcessParentNotFoundError,
  ProcessParentMismatchedError,
  MemoryLimitError,
  MemoryNotMappedError,
  _IpcNotReceivingError,
  InternalError,
}

impl core::convert::From<crate::mm::page_pool::Error> for Error {
  fn from(e: crate::mm::page_pool::Error) -> Self {
    match e {
      crate::mm::page_pool::Error::OutOfFrameError => { OutOfMemoryError }
      _ => { InternalError }
    }
  }
}

impl core::convert::From<crate::lib::page_table::Error> for Error {
  fn from(_: crate::lib::page_table::Error) -> Self {
    InternalError
  }
}

impl core::convert::From<crate::lib::process::Error> for Error {
  fn from(e: crate::lib::process::Error) -> Self {
    match e {
      _ => { InternalError }
    }
  }
}

pub trait SystemCallTrait {
  fn putc(c: char);
  fn get_pid() -> u16;
  fn get_tid() -> u16;
  fn thread_yield();
  fn process_destroy(pid: u16) -> Result<(), Error>;
  fn process_set_exception_handler(pid: u16, value: usize, sp: usize) -> Result<(), Error>;
  fn mem_alloc(pid: u16, va: usize, perm: usize) -> Result<(), Error>;
  fn mem_map(src_pid: u16, src_va: usize, dst_pid: u16, dst_va: usize, perm: usize) -> Result<(), Error>;
  fn mem_unmap(pid: u16, va: usize) -> Result<(), Error>;
  fn process_alloc() -> Result<u16, Error>;
  fn thread_alloc() -> Result<u16, Error>;
  fn thread_set_status(pid: u16, status: crate::lib::thread::Status) -> Result<(), Error>;
  fn ipc_receive(dst_va: usize);
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, perm: usize) -> Result<(), Error>;
}

pub struct SystemCall;

fn lookup_pid(pid: u16, check_parent: bool) -> Result<Process, Error> {
  if pid == 0 {
    match current_process() {
      None => { Err(InternalError) }
      Some(p) => { Ok(p) }
    }
  } else {
    if let Some(p) = crate::lib::process::lookup(pid) {
      if check_parent {
        if let Some(parent) = p.parent() {
          match current_process() {
            None => { Err(InternalError) }
            Some(current) => {
              if current.pid() == parent.pid() {
                Ok(p)
              } else {
                Err(ProcessParentMismatchedError)
              }
            }
          }
        } else {
          Err(ProcessParentNotFoundError)
        }
      } else {
        Ok(p)
      }
    } else {
      Err(ProcessPidNotFoundError)
    }
  }
}

impl SystemCallTrait for SystemCall {
  fn putc(c: char) {
    crate::driver::uart::putc(c as u8);
  }

  fn get_pid() -> u16 {
    match current_process() {
      None => { 0 }
      Some(p) => { p.pid() }
    }
  }

  fn get_tid() -> u16 {
    match current_thread() {
      None => { 0 }
      Some(t) => { t.tid() }
    }
  }

  fn thread_yield() {
    crate::lib::scheduler::schedule();
  }

  fn process_destroy(pid: u16) -> Result<(), Error> {
    let p = lookup_pid(pid, true)?;
    p.destroy();
    Ok(())
  }

  fn process_set_exception_handler(pid: u16, entry: usize, stack_top: usize) -> Result<(), Error> {
    let p = lookup_pid(pid, true)?;
    if entry >= CONFIG_USER_LIMIT || stack_top >= CONFIG_USER_LIMIT || stack_top % PAGE_SIZE != 0 {
      return Err(InvalidArgumentError);
    }
    p.set_exception_handler(entry, stack_top);
    Ok(())
  }

  fn mem_alloc(pid: u16, va: usize, attr: usize) -> Result<(), Error> {
    if va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let p = lookup_pid(pid, true)?;
    let frame = crate::mm::page_pool::try_alloc()?;
    frame.zero();
    let page_table = p.page_table();
    let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
    let attr = user_attr.filter();
    page_table.insert_page(va, frame, attr)?;
    Ok(())
  }

  fn mem_map(src_pid: u16, src_va: usize, dst_pid: u16, dst_va: usize, attr: usize) -> Result<(), Error> {
    let src_va = round_down(src_va, PAGE_SIZE);
    let dst_va = round_down(dst_va, PAGE_SIZE);
    if src_va >= CONFIG_USER_LIMIT || dst_va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let src_pid = lookup_pid(src_pid, true)?;
    let dst_pid = lookup_pid(dst_pid, true)?;
    let src_pt = src_pid.page_table();
    if let Some(pte) = src_pt.lookup_page(src_va) {
      let pa = pte.pa();
      let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
      let attr = user_attr.filter();
      let dst_pt = dst_pid.page_table();
      dst_pt.insert_page(dst_va, crate::mm::PageFrame::new(pa), attr)?;
      Ok(())
    } else {
      Err(MemoryNotMappedError)
    }
  }

  fn mem_unmap(pid: u16, va: usize) -> Result<(), Error> {
    if va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let p = lookup_pid(pid, true)?;
    let page_table = p.page_table();
    page_table.remove_page(va)?;
    Ok(())
  }

  fn process_alloc() -> Result<Pid, Error> {
    let t = current_thread().unwrap();
    let p = t.process().unwrap();
    let child = crate::lib::process::alloc(Some(p));
    let mut ctx = crate::lib::current_core().context();
    ctx.set_syscall_return_value(0);
    let child_thread = crate::lib::thread::alloc_user(0, 0, 0, child);
    *child_thread.context() = ctx;
    child_thread.set_status(crate::lib::thread::Status::TsNotRunnable);
    child.set_main_thread(child_thread);
    Ok(child.pid())
  }

  fn thread_alloc() -> Result<u16, Error> {
    unimplemented!()
  }

  fn thread_set_status(pid: u16, status: crate::lib::thread::Status) -> Result<(), Error> {
    use crate::lib::thread::Status::{TsRunnable, TsNotRunnable};
    if status != TsRunnable && status != TsNotRunnable {
      return Err(InvalidArgumentError);
    }
    let p = lookup_pid(pid, true)?;
    p.main_thread().set_status(status);
    Ok(())
  }

  #[allow(unused_variables)]
  fn ipc_receive(dst_va: usize) {
    unimplemented!()
    // unsafe {
    //   let t = current_thread().unwrap();
    //   let p = t.process().unwrap();
    //   (*p.ipc()).attribute = dst_va;
    //   (*p.ipc()).receiving = true;
    //   SystemCall::thread_yield();
    // }
  }

  #[allow(unused_variables)]
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, attr: usize) -> Result<(), Error> {
    unimplemented!()
    // if src_va >= CONFIG_USER_LIMIT {
    //   return Err(MemoryLimitError);
    // }
    // unsafe {
    //   let t = current_thread();
    //   let src_p = t.process().unwrap();
    //   let dst_p = lookup_pid(pid, false)?;
    //   if !(*dst_p.ipc()).receiving {
    //     return Err(IpcNotReceivingError);
    //   }
    //   if src_va != 0 {
    //     let dst_va = (*dst_p.ipc()).address;
    //     if dst_va >= CONFIG_USER_LIMIT {
    //       return Err(MemoryLimitError);
    //     }
    //     let src_page_table = src_p.page_table();
    //     if let Some(src_pte) = src_page_table.lookup_page(src_va) {
    //       let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
    //       let attr = user_attr.filter();
    //       (*dst_p.ipc()).attribute = ArchPageTableEntry::from(Entry::new(attr, 0)).to_pte();
    //       let dst_page_table = dst_p.page_table();
    //       dst_page_table.insert_page(dst_va, PageFrame::new(src_pte.pa()), attr)?;
    //     } else {
    //       return Err(InvalidArgumentError);
    //     }
    //   }
    //   (*dst_p.ipc()).receiving = false;
    //   (*dst_p.ipc()).from = src_p.pid();
    //   (*dst_p.ipc()).value = value;
    //   dst_p.main_thread().set_status(crate::lib::thread::Status::TsRunnable);
    //   Ok(())
    // }
  }
}