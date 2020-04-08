use lib::process::*;
use lib::syscall::SystemCallError::*;
use arch::*;
use lib::process_pool::ProcessPoolError;
use config::*;

pub enum SystemCallError {
  ScePidNotFound = 1,
  ScePidNoParent = 2,
  ScePidParentNotMatched = 3,
  SceMemVaExceedLimit = 4,
  SceMemSrcVaNotMapped = 5,
  SceMemPageTableError = 6,
  SceProcessDirectoryNone = 7,
  SceCurrentProcessNone = 8,
  SceProcessPoolError = 9,
  SceIpcNotReceiving = 10,
}

impl core::convert::From<PageTableError> for SystemCallError {
  fn from(_: PageTableError) -> Self {
    SceMemPageTableError
  }
}

impl core::convert::From<ProcessPoolError> for SystemCallError {
  fn from(_: ProcessPoolError) -> Self {
    SceProcessPoolError
  }
}

pub trait SystemCallImpl {
  fn putc(c : char);
  fn getpid() -> u16;
  fn process_yield();
  fn process_destroy(pid: u16) -> Result<(), SystemCallError>;
  fn process_set_exception_handler(pid: u16, value: usize, sp: usize) -> Result<(), SystemCallError>;
  fn mem_alloc(pid: u16, va: usize, perm: usize) -> Result<(), SystemCallError>;
  fn mem_map(src_pid: u16, src_va: usize, dst_pid: u16, dst_va: usize, perm: usize) -> Result<(), SystemCallError>;
  fn mem_unmap(pid: u16, va: usize) -> Result<(), SystemCallError>;

  fn process_alloc() -> Result<u16, SystemCallError>;
  fn process_set_status(pid: u16, status: super::process::ProcessStatus) -> Result<(), SystemCallError>;

  fn ipc_receive(dst_va: usize);
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, perm: usize) -> Result<(), SystemCallError>;
}

pub struct SystemCall;

fn lookup_pid(pid: u16, check_parent: bool) -> Result<Pid, SystemCallError> {
  use lib::process_pool::*;
  if pid == 0 {
    unsafe {
      Ok(CURRENT_PROCESS.ok_or_else(|| SceCurrentProcessNone)?)
    }
  } else {
    if let Some(p) = lookup(pid) {
      if check_parent {
        if let Some(parent) = p.parent(){
          unsafe {
            if CURRENT_PROCESS.ok_or_else(|| SceCurrentProcessNone)?.pid() == parent.pid() {
              Ok(p)
            } else {
              Err(ScePidParentNotMatched)
            }
          }
        } else {
          Err(ScePidNoParent)
        }
      } else {
        Ok(p)
      }
    } else {
      Err(ScePidNotFound)
    }
  }
}

impl SystemCallImpl for SystemCall {
  fn putc(c: char) {
    crate::driver::uart::uart_putc(c as u8);
  }

  fn getpid() -> u16 {
    unsafe {
      CURRENT_PROCESS.unwrap().pid()
    }
  }

  fn process_yield() {
    use lib::scheduler::{SCHEDULER, Scheduler};
    use lib::process_pool::pid_list;
    unsafe {
      SCHEDULER.schedule(pid_list());
    }
  }

  fn process_destroy(pid: u16) -> Result<(), SystemCallError> {
    let p = lookup_pid(pid, true)?;
    p.destroy();
    Ok(())
  }

  fn process_set_exception_handler(pid: u16, value: usize, sp: usize) -> Result<(), SystemCallError> {
    let p = lookup_pid(pid, true)?;
    unsafe {
      (*p.pcb()).exception_handler = value;
      (*p.pcb()).exception_stack_top = sp;
    }
    Ok(())
  }

  fn mem_alloc(pid: u16, va: usize, attr: usize) -> Result<(), SystemCallError> {
    if va >= CONFIG_MEM_USER_LIMIT {
      return Err(SceMemVaExceedLimit);
    }
    let p = lookup_pid(pid, true)?;
    let frame = crate::mm::page_pool::alloc();
    frame.zero();
    unsafe {
      let page_table = (*p.pcb()).directory.ok_or_else(|| SceProcessDirectoryNone)?;
      let attr = PageTableEntry::from(ArchPageTableEntry::new(attr as u64)).attr;
      page_table.insert_page(va, frame, attr);
    }
    Ok(())
  }

  fn mem_map(src_pid: u16, src_va: usize, dst_pid: u16, dst_va: usize, attr: usize) -> Result<(), SystemCallError> {
    let src_va = round_down(src_va, PAGE_SIZE);
    let dst_va = round_down(dst_va, PAGE_SIZE);
    if dst_va >= CONFIG_MEM_USER_LIMIT {
      return Err(SceMemVaExceedLimit);
    }
    let src_pid = lookup_pid(src_pid, true)?;
    let dst_pid = lookup_pid(dst_pid, true)?;
    unsafe {
      let src_pt = (*src_pid.pcb()).directory.ok_or_else(|| SceProcessDirectoryNone)?;
      if let Some(pte) = src_pt.lookup_page(src_va) {
        let pa = pte.addr;
        let attr = PageTableEntry::from(ArchPageTableEntry::new(attr as u64)).attr;
        //println!("map from [{}]@{:08x} to [{}]@{:08x} attr: {:?}", src_pid.pid(), src_va, dst_pid.pid(), dst_va, attr);
        let dst_pt = (*dst_pid.pcb()).directory.ok_or_else(|| SceProcessDirectoryNone)?;
        dst_pt.insert_page(dst_va, crate::mm::PageFrame::new(pa), attr)?;
        Ok(())
      } else {
        Err(SceMemSrcVaNotMapped)
      }
    }
  }

  fn mem_unmap(pid: u16, va: usize) -> Result<(), SystemCallError> {
    let p = lookup_pid(pid, true)?;
    unsafe {
      let page_table = (*p.pcb()).directory.ok_or_else(|| SceProcessDirectoryNone)?;
      page_table.remove_page(va)?;
    }
    Ok(())
  }

  fn process_alloc() -> Result<u16, SystemCallError> {
    use lib::*;
    unsafe {
      let p = CURRENT_PROCESS.unwrap();
      let child = process_pool::alloc(Some(p), 0)?;
      let mut ctx = CONTEXT_FRAME.clone();
      ctx.set_argument(0);
      (*child.pcb()).context = Some(ctx);
      // TODO: maybe need a workaround for copy on write of stack frame
      (*child.pcb()).status = ProcessStatus::PsNotRunnable;
      Ok(child.pid())
    }
  }

  fn process_set_status(pid: u16, status: ProcessStatus) -> Result<(), SystemCallError> {
    let p = lookup_pid(pid, true)?;
    unsafe {
      (*p.pcb()).status = status;
    }
    Ok(())
  }

  fn ipc_receive(dst_va: usize) {
    unsafe {
      let p = CURRENT_PROCESS.unwrap();
      (*p.pcb()).ipc_dst_attr = dst_va;
      (*p.pcb()).ipc_receiving = true;
      (*p.pcb()).status = ProcessStatus::PsNotRunnable;
      SystemCall::process_yield();
    }
  }

  fn ipc_can_send(pid: u16, value: usize, src_va: usize, attr: usize) -> Result<(), SystemCallError> {
    let p = lookup_pid(pid, false)?;
    unsafe {
      if !(*p.pcb()).ipc_receiving {
        return Err(SceIpcNotReceiving);
      }
      (*p.pcb()).ipc_receiving = false;
      (*p.pcb()).ipc_from = Some(CURRENT_PROCESS.unwrap());
      (*p.pcb()).ipc_value = value;
      (*p.pcb()).status = ProcessStatus::PsRunnable;
      Ok(())
    }
  }
}