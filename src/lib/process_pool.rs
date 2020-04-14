use alloc::vec::Vec;

use spin::Mutex;

use crate::config::*;
use crate::lib::process::{Pid, Process, Status};

struct ProcessPool {
  free: Vec<Process>,
  alloced: Vec<Process>, // Runnable and NotRunnable
}

pub enum Error {
  OutOfProcessError,
  ProcessNotFoundError,
}

impl ProcessPool {
  fn init(&mut self) {
    for index in 0..CONFIG_PROCESS_NUMBER {
      let pid = (index + 1) as Pid;
      let p = Process::new(pid);
      unsafe {
        (*p.pcb()).id = pid;
        (*p.ipc()).id = pid;
      }
      self.free.push(p);
    }
  }

  fn alloc(&mut self, parent: Option<Process>, arg: usize) -> Result<Process, Error> {
    use crate::arch::{ContextFrame, ContextFrameTrait};
    unsafe {
      if let Some(p) = self.free.pop() {
        p.setup_vm();
        (*p.pcb()).parent = parent;
        (*p.pcb()).status = Status::PsRunnable;
        let mut ctx = ContextFrame::default();
        ctx.set_argument(arg);
        ctx.set_stack_pointer(CONFIG_USER_STACK_TOP);
        (*p.pcb()).context = Some(ctx);
        self.alloced.push(p);
        Ok(p)
      } else {
        Err(Error::OutOfProcessError)
      }
    }
  }

  fn free(&mut self, p: Process) -> Result<(), Error> {
    if let Some(p) = self.alloced.remove_item(&p) {
      unsafe {
        (*p.pcb()).parent = None;
        (*p.pcb()).page_table = None;
        (*p.pcb()).context = None;
        (*p.pcb()).status = Status::PsFree;
        (*p.pcb()).exception_handler = 0;
        (*p.pcb()).exception_stack_top = 0;
        (*p.ipc()).from = 0;
        (*p.ipc()).receiving = false;
        (*p.ipc()).value = 0;
        (*p.ipc()).address = 0;
        (*p.ipc()).attribute = 0;
        self.free.push(p);
      }
      return Ok(());
    } else {
      Err(Error::ProcessNotFoundError)
    }
  }

  fn pid_list(&self) -> Vec<Process> {
    self.alloced.clone()
  }

  fn lookup(&self, p: Process) -> Option<Process> {
    for i in self.alloced.iter() {
      if i.pid() == p.pid() {
        return Some(i.clone());
      }
    }
    None
  }
}

static PROCESS_POOL: Mutex<ProcessPool> = Mutex::new(ProcessPool {
  free: Vec::new(),
  alloced: Vec::new(),
});

pub fn init() {
  let mut pool = PROCESS_POOL.lock();
  pool.init();
  drop(pool);
}

pub fn alloc(parent: Option<Process>, arg: usize) -> Result<Process, Error> {
  let mut pool = PROCESS_POOL.lock();
  let r = pool.alloc(parent, arg);
  drop(pool);
  r
}

pub fn free(p: Process) {
  let mut pool = PROCESS_POOL.lock();
  match pool.free(p) {
    Ok(_) => {}
    Err(_) => { println!("process_pool: free: process not found") }
  }
  drop(pool);
}

pub fn pid_list() -> Vec<Process> {
  let pool = PROCESS_POOL.lock();
  let r = pool.pid_list();
  drop(pool);
  r
}

pub fn lookup(p: Process) -> Option<Process> {
  let pool = PROCESS_POOL.lock();
  let r = pool.lookup(p);
  drop(pool);
  r
}