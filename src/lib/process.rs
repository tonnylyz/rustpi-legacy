use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::Mutex;

use crate::arch::{PAGE_SIZE, PageTable};
use crate::config::CONFIG_USER_STACK_TOP;
use crate::lib::bitmap::BitMap;
use crate::lib::current_thread;
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::lib::thread::Thread;

pub type Pid = u16;

#[derive(Debug)]
pub struct ControlBlock {
  pid: Pid,
  threads: Mutex<Vec<Thread>>,
  parent: Option<Process>,
  page_table: PageTable,
  exception_handler: Mutex<Option<(usize, usize)>>,
}


#[derive(Debug, Clone)]
pub struct Process(Arc<ControlBlock>);

impl PartialEq for Process {
  fn eq(&self, other: &Self) -> bool {
    self.0.pid == other.0.pid
  }
}

impl Process {
  pub fn pid(&self) -> Pid {
    self.0.pid
  }

  pub fn main_thread(&self) -> Thread {
    let lock = self.0.threads.lock();
    let r = lock[0].clone();
    drop(lock);
    r
  }

  pub fn set_main_thread(&self, t: Thread) {
    let mut lock = self.0.threads.lock();
    assert!(lock.is_empty());
    lock.push(t);
    drop(lock);
  }

  pub fn exception_handler(&self) -> Option<(usize, usize)> {
    let lock = self.0.exception_handler.lock();
    let r = *lock;
    drop(lock);
    r
  }

  pub fn set_exception_handler(&self, entry: usize, stack_top: usize) {
    let mut lock = self.0.exception_handler.lock();
    *lock = Some((entry, stack_top));
    drop(lock);
  }

  pub fn page_table(&self) -> PageTable {
    self.0.page_table
  }

  pub fn parent(&self) -> Option<Process> {
    match &self.0.parent {
      None => { None }
      Some(p) => { Some(p.clone()) }
    }
  }

  pub fn destroy(&self) {
    for t in self.0.threads.lock().iter() {
      t.destroy();
    }
    self.0.page_table.destroy();
    let frame = self.0.page_table.directory();
    crate::mm::page_pool::decrease_rc(frame);
    free(self);
    if current_thread().is_none() {
      crate::lib::scheduler::schedule();
    }
  }
}


struct ProcessPool {
  bitmap: BitMap,
  alloced: Vec<Process>,
}

pub enum Error {
  ProcessNotFoundError,
}

fn make_user_page_table() -> PageTable {
  let frame = crate::mm::page_pool::alloc();
  crate::mm::page_pool::increase_rc(frame);
  let page_table = PageTable::new(frame);
  page_table.recursive_map(crate::config::CONFIG_RECURSIVE_PAGE_TABLE_BTM);
  page_table
}

impl ProcessPool {
  fn alloc(&mut self, parent: Option<Process>) -> Process {
    let id = self.bitmap.alloc() as Pid;
    let arc = Arc::new(ControlBlock {
      pid: id,
      threads: Mutex::new(Vec::new()),
      parent,
      page_table: make_user_page_table(),
      exception_handler: Mutex::new(None),
    });
    let mut map = PROCESS_MAP.lock();
    map.insert(id, arc.clone());
    drop(map);
    self.alloced.push(Process(arc.clone()));
    Process(arc)
  }

  fn free(&mut self, p: &Process) -> Result<(), Error> {
    if let Some(p) = self.alloced.remove_item(p) {
      let mut map = PROCESS_MAP.lock();
      map.remove(&p.pid());
      drop(map);
      self.bitmap.clear(p.pid() as usize);
      Ok(())
    } else {
      Err(Error::ProcessNotFoundError)
    }
  }

  #[allow(dead_code)]
  fn list(&self) -> Vec<Process> {
    self.alloced.clone()
  }
}

lazy_static! {
  static ref PROCESS_MAP: Mutex<BTreeMap<Pid, Arc<ControlBlock>>> = Mutex::new(BTreeMap::new());
}

static PROCESS_POOL: Mutex<ProcessPool> = Mutex::new(ProcessPool {
  bitmap: BitMap::new(),
  alloced: Vec::new(),
});

pub fn alloc(parent: Option<Process>) -> Process {
  let mut pool = PROCESS_POOL.lock();
  let r = pool.alloc(parent);
  drop(pool);
  r
}

pub fn free(p: &Process) {
  let mut pool = PROCESS_POOL.lock();
  match pool.free(p) {
    Ok(_) => {}
    Err(_) => { println!("process_pool: free: process not found") }
  }
  drop(pool);
}

#[allow(dead_code)]
pub fn list() -> Vec<Process> {
  let pool = PROCESS_POOL.lock();
  let r = pool.list();
  drop(pool);
  r
}

pub fn lookup(pid: Pid) -> Option<Process> {
  let map = PROCESS_MAP.lock();
  let r = match map.get(&pid) {
    Some(arc) => Some(Process(arc.clone())),
    None => None
  };
  drop(map);
  r
}

pub fn create(elf: &'static [u8], arg: usize) {
  let p = alloc(None);
  let page_table = p.page_table();
  match crate::lib::elf::load(elf, page_table) {
    Ok(pc) => {
      let sp = CONFIG_USER_STACK_TOP;
      match page_table.insert_page(sp - PAGE_SIZE, crate::mm::page_pool::alloc(), EntryAttribute::user_default()) {
        Ok(_) => {}
        Err(_) => { panic!("process: create: page_table.insert_page failed") }
      }
      let t = crate::lib::thread::alloc_user(pc, sp, arg, p.clone());
      t.set_status(crate::lib::thread::Status::TsRunnable);
      p.set_main_thread(t);
    }
    Err(_) => { panic!("process: create: load err") }
  }
}