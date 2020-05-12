use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::borrow::BorrowMut;

use spin::Mutex;

use crate::arch::*;
use crate::lib::bitmap::BitMap;
use crate::lib::current_thread;
use crate::lib::page_table::PageTableTrait;
use crate::lib::process::Process;

pub type Tid = u16;

pub enum Type {
  User(Process),
  Kernel,
}

#[derive(Eq, PartialEq)]
pub enum Status {
  TsRunnable = 1,
  TsNotRunnable = 2,
}

pub struct ControlBlock {
  #[allow(dead_code)]
  tid: u16,
  t: Type,
  status: Status,
  context: ContextFrame,
}

pub enum Error {
  ThreadNotFoundError
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Thread(Tid);

impl Thread {
  pub fn tid(&self) -> Tid {
    self.0
  }

  pub fn set_status(&self, status: Status) {
    self.tcb().status = status;
  }

  pub fn runnable(&self) -> bool {
    self.tcb().status == Status::TsRunnable
  }

  pub fn process(&self) -> Option<Process> {
    match self.tcb().t {
      Type::User(p) => {
        Some(p)
      }
      _ => {
        None
      }
    }
  }

  pub fn tcb(&self) -> &mut ControlBlock {
    let mut map = THREAD_MAP.lock();
    let r;
    match map.get_mut(&self.0) {
      Some(b) => {
        r = Box::borrow_mut(b) as *mut ControlBlock
      }
      None => panic!("thread: tcb missing")
    }
    drop(map);
    unsafe { r.as_mut().unwrap() }
  }

  pub fn context(&self) -> &mut ContextFrame {
    unsafe {
      let tcb = self.tcb();
      (&mut tcb.context as *mut ContextFrame).as_mut().unwrap()
    }
  }

  pub fn run(&self) {
    println!("run thread {}", self.tid());
    let core = crate::arch::common::core::current();
    if let Some(t) = current_thread() {
      // Note: normal switch
      t.tcb().context = core.context();
      core.install_context(&self.context());
    } else {
      if core.has_context() {
        // Note: previous process has been destroyed
        core.install_context(&self.context());
      } else {
        // Note: this is first run
        // Arch::start_first_process prepare the context to stack
      }
    }
    (*core).set_running_thread(Some(self.clone()));
    if let Some(p) = self.process() {
      println!("run process {}", self.process().unwrap().pid());
      crate::arch::PageTable::set_user_page_table(p.page_table(), p.pid() as AddressSpaceId);
    }
    crate::arch::Arch::invalidate_tlb();
  }
}

struct ThreadPool {
  bitmap: BitMap,
  alloced: Vec<Thread>,
}

impl ThreadPool {
  fn alloc_user(&mut self, pc: usize, sp: usize, arg: usize, p: Process) -> Thread {
    let id = self.bitmap.alloc() as Tid;
    let b = Box::new(ControlBlock {
      tid: id,
      t: Type::User(p),
      status: Status::TsNotRunnable,
      context: ContextFrame::new(pc, sp, arg, false),
    });
    let mut map = THREAD_MAP.lock();
    map.insert(id, b);
    drop(map);
    self.alloced.push(Thread(id));
    Thread(id)
  }

  fn alloc_kernel(&mut self, pc: usize, sp: usize, arg: usize) -> Thread {
    let id = self.bitmap.alloc() as Tid;
    let b = Box::new(ControlBlock {
      tid: id,
      t: Type::Kernel,
      status: Status::TsNotRunnable,
      context: ContextFrame::new(pc, sp, arg, true),
    });
    let mut map = THREAD_MAP.lock();
    map.insert(id, b);
    drop(map);
    self.alloced.push(Thread(id));
    Thread(id)
  }

  fn free(&mut self, t: Thread) -> Result<(), Error> {
    if let Some(t) = self.alloced.remove_item(&t) {
      // TODO: free box in map
      self.bitmap.clear(t.0 as usize);
      Ok(())
    } else {
      Err(Error::ThreadNotFoundError)
    }
  }

  fn list(&self) -> Vec<Thread> {
    self.alloced.clone()
  }

  fn lookup(&self, tid: Tid) -> Option<Thread> {
    for i in self.alloced.iter() {
      if i.tid() == tid {
        return Some(i.clone());
      }
    }
    None
  }
}

lazy_static! {
  static ref THREAD_MAP: Mutex<BTreeMap<Tid, Box<ControlBlock>>> = Mutex::new(BTreeMap::new());
}

static THREAD_POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool {
  bitmap: BitMap::new(),
  alloced: Vec::new(),
});

pub fn alloc_user(pc: usize, sp: usize, arg: usize, p: Process) -> Thread {
  let mut pool = THREAD_POOL.lock();
  let r = pool.alloc_user(pc, sp, arg, p);
  drop(pool);
  r
}

pub fn alloc_kernel(pc: usize, sp: usize, arg: usize) -> Thread {
  let mut pool = THREAD_POOL.lock();
  let r = pool.alloc_kernel(pc, sp, arg);
  drop(pool);
  r
}

pub fn free(p: Thread) {
  let mut pool = THREAD_POOL.lock();
  match pool.free(p) {
    Ok(_) => {}
    Err(_) => { println!("process_pool: free: process not found") }
  }
  drop(pool);
}

pub fn list() -> Vec<Thread> {
  let pool = THREAD_POOL.lock();
  let r = pool.list();
  drop(pool);
  r
}

pub fn lookup(tid: Tid) -> Option<Thread> {
  let pool = THREAD_POOL.lock();
  let r = pool.lookup(tid);
  drop(pool);
  r
}
