use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::{Mutex, MutexGuard};

use crate::arch::{AddressSpaceId, ArchTrait, ContextFrame, ContextFrameTrait, CoreId, CoreTrait};
use crate::lib::bitmap::BitMap;
use crate::lib::current_thread;
use crate::lib::page_table::PageTableTrait;
use crate::lib::process::Process;

pub type Tid = u16;

#[derive(Debug)]
pub enum Type {
  User(Process),
  Kernel,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Status {
  TsRunnable = 1,
  TsNotRunnable = 2,
  TsIdle = 3,
}

#[derive(Debug)]
pub struct ControlBlock {
  tid: u16,
  t: Type,
  status: Mutex<Status>,
  context: Mutex<ContextFrame>,
  core: Mutex<Option<CoreId>>,
}

pub enum Error {
  ThreadNotFoundError
}

#[derive(Debug, Clone)]
pub struct Thread(Arc<ControlBlock>);

impl PartialEq for Thread {
  fn eq(&self, other: &Self) -> bool {
    self.0.tid == other.0.tid
  }
}

impl Thread {
  pub fn tid(&self) -> Tid {
    self.0.tid
  }

  pub fn set_status(&self, status: Status) {
    let mut lock = self.0.status.lock();
    *lock = status;
    drop(lock);
  }

  pub fn runnable(&self) -> bool {
    let lock = self.0.status.lock();
    let r = *lock == Status::TsRunnable;
    drop(lock);
    r
  }

  pub fn process(&self) -> Option<Process> {
    match &self.0.t {
      Type::User(p) => {
        Some(p.clone())
      }
      _ => {
        None
      }
    }
  }

  pub fn context(&self) -> MutexGuard<ContextFrame> {
    self.0.context.lock()
  }

  pub fn run(&self) -> bool {
    let mut core_lock = self.0.core.lock();
    match *core_lock {
      Some(core_id) => {
        if core_id != crate::arch::Arch::core_id() {
          drop(core_lock);
          return false;
        }
      }
      None => {
        *core_lock = Some(crate::arch::Arch::core_id());
      }
    }
    drop(core_lock);
    let core = crate::lib::core::current();
    if let Some(t) = current_thread() {
      // Note: normal switch
      let mut old = t.context();
      *old = *core.context();
      drop(old);
      let new = self.context();
      *core.context_mut() = *new;
      drop(new);
    } else {
      if core.has_context() {
        // Note: previous process has been destroyed
        let new = self.context();
        *core.context_mut() = *new;
        drop(new);
      } else {
        // Note: this is first run
        // `main` prepare the context to stack
      }
    }
    core.set_running_thread(Some(self.clone()));
    if let Some(p) = self.process() {
      println!("\ncore_{} switch to P{}/T{}", crate::arch::Arch::core_id(), p.pid(), self.tid());
      crate::arch::PageTable::set_user_page_table(p.page_table(), p.pid() as AddressSpaceId);
    }
    crate::arch::Arch::invalidate_tlb();
    true
  }

  pub fn destroy(&self) {
    if let Some(t) = current_thread() {
      if self.0.tid == t.tid() {
        crate::lib::core::current().set_running_thread(None);
      }
    }
    free(self)
  }
}

struct ThreadPool {
  bitmap: BitMap,
  alloced: Vec<Thread>,
}

impl ThreadPool {
  fn alloc_user(&mut self, pc: usize, sp: usize, arg: usize, p: Process) -> Thread {
    let id = self.bitmap.alloc() as Tid;
    let arc = Arc::new(ControlBlock {
      tid: id,
      t: Type::User(p),
      status: Mutex::new(Status::TsNotRunnable),
      context: Mutex::new(ContextFrame::new(pc, sp, arg, false)),
      core: Mutex::new(None),
    });
    let mut map = THREAD_MAP.lock();
    map.insert(id, arc.clone());
    drop(map);
    self.alloced.push(Thread(arc.clone()));
    Thread(arc)
  }

  fn alloc_kernel(&mut self, pc: usize, sp: usize, arg: usize) -> Thread {
    let id = self.bitmap.alloc() as Tid;
    let arc = Arc::new(ControlBlock {
      tid: id,
      t: Type::Kernel,
      status: Mutex::new(Status::TsNotRunnable),
      context: Mutex::new(ContextFrame::new(pc, sp, arg, true)),
      core: Mutex::new(None),
    });
    let mut map = THREAD_MAP.lock();
    map.insert(id, arc.clone());
    drop(map);
    self.alloced.push(Thread(arc.clone()));
    Thread(arc)
  }

  fn free(&mut self, t: &Thread) -> Result<(), Error> {
    if let Some(t) = self.alloced.remove_item(t) {
      let mut map = THREAD_MAP.lock();
      map.remove(&t.tid());
      drop(map);
      self.bitmap.clear(t.tid() as usize);
      Ok(())
    } else {
      Err(Error::ThreadNotFoundError)
    }
  }

  fn list(&self) -> Vec<Thread> {
    self.alloced.clone()
  }
}

lazy_static! {
  static ref THREAD_MAP: Mutex<BTreeMap<Tid, Arc<ControlBlock>>> = Mutex::new(BTreeMap::new());
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

pub fn free(t: &Thread) {
  let mut pool = THREAD_POOL.lock();
  match pool.free(t) {
    Ok(_) => {}
    Err(_) => { println!("thread_pool: free: thread not found") }
  }
  drop(pool);
}

pub fn list() -> Vec<Thread> {
  let pool = THREAD_POOL.lock();
  let r = pool.list();
  drop(pool);
  r
}

#[allow(dead_code)]
pub fn lookup(tid: Tid) -> Option<Thread> {
  let map = THREAD_MAP.lock();
  let r = match map.get(&tid) {
    Some(arc) => Some(Thread(arc.clone())),
    None => None
  };
  drop(map);
  r
}
