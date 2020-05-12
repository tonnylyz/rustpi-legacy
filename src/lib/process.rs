use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::borrow::BorrowMut;
use core::fmt::{Display, Formatter};

use spin::Mutex;

use crate::arch::*;
use crate::config::*;
use crate::lib::bitmap::BitMap;
use crate::lib::current_thread;
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::lib::thread::Thread;

pub type Pid = u16;

#[repr(C, align(32))]
#[derive(Copy, Clone, Debug)]
pub struct Ipc {
  pub id: Pid,
  pub from: Pid,
  pub receiving: bool,
  pub value: usize,
  pub address: usize,
  pub attribute: usize,
}

#[no_mangle]
#[link_section = ".bss.ipc"]
pub static mut IPC_LIST: [Ipc; CONFIG_PROCESS_NUMBER] = [Ipc {
  id: 0,
  from: 0,
  receiving: false,
  value: 0,
  address: 0,
  attribute: 0,
}; CONFIG_PROCESS_NUMBER];

#[derive(Debug)]
pub struct ControlBlock {
  pid: Pid,
  threads: Vec<Thread>,
  parent: Option<Process>,
  page_table: PageTable,
  exception_handler: Option<(usize, usize)>,
}


impl Display for ControlBlock {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    writeln!(f, "Process {}", self.pid)?;
    Ok(())
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Process(Pid);

impl Process {
  pub fn pid(&self) -> Pid {
    self.0
  }

  pub fn main_thread(&self) -> Thread {
    self.pcb().threads.get(0).unwrap().clone()
  }

  pub fn set_main_thread(&self, t: Thread) {
    assert!(self.pcb().threads.is_empty());
    self.pcb().threads.push(t);
  }

  pub fn exception_handler(&self) -> Option<(usize, usize)> {
    self.pcb().exception_handler
  }

  pub fn set_exception_handler(&self, entry: usize, stack_top: usize) {
    self.pcb().exception_handler = Some((entry, stack_top));
  }

  pub fn page_table(&self) -> PageTable {
    self.pcb().page_table
  }

  pub fn pcb(&self) -> &mut ControlBlock {
    let mut map = PROCESS_MAP.lock();
    let r;
    match map.get_mut(&self.0) {
      Some(b) => {
        r = Box::borrow_mut(b) as *mut ControlBlock
      }
      None => panic!("process: pcb missing")
    }
    drop(map);
    unsafe {
      r.as_mut().unwrap()
    }
  }

  #[allow(dead_code)]
  pub fn ipc(&self) -> &mut Ipc {
    unsafe {
      ((&mut IPC_LIST[self.0 as usize]) as *mut Ipc).as_mut().unwrap()
    }
  }

  pub fn parent(&self) -> Option<Process> {
    self.pcb().parent
  }

  pub fn free(self) {
    self.pcb().page_table.destroy();
    let frame = self.pcb().page_table.directory();
    crate::mm::page_pool::decrease_rc(frame);
    super::process::free(self);
  }

  pub fn destroy(&self) {
    self.free();
    if let Some(t) = current_thread() {
      if let Some(p) = t.process() {
        if p.0 == self.0 {
          crate::arch::common::core::current().set_running_thread(None);
          crate::lib::scheduler::schedule();
        }
      }
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
  page_table.recursive_map(CONFIG_RECURSIVE_PAGE_TABLE_BTM);
  for i in 0..(CONFIG_PROCESS_IPC_SIZE * CONFIG_PROCESS_NUMBER / PAGE_SIZE) {
    unsafe {
      let va = CONFIG_USER_IPC_LIST_BTM + i * PAGE_SIZE;
      let pa = (&IPC_LIST[i * (PAGE_SIZE / CONFIG_PROCESS_IPC_SIZE)] as *const Ipc as usize).kva2pa();
      page_table.map(va, pa, EntryAttribute::user_readonly());
    }
  }
  page_table
}

impl ProcessPool {
  fn alloc(&mut self, parent: Option<Process>) -> Process {
    let id = self.bitmap.alloc() as Pid;
    let b = Box::new(ControlBlock {
      pid: id,
      threads: Vec::new(),
      parent,
      page_table: make_user_page_table(),
      exception_handler: None,
    });
    let mut map = PROCESS_MAP.lock();
    map.insert(id, b);
    drop(map);
    self.alloced.push(Process(id));
    Process(id)
  }

  fn free(&mut self, p: Process) -> Result<(), Error> {
    if let Some(p) = self.alloced.remove_item(&p) {
      self.bitmap.clear(p.0 as usize);
      // TODO: free box in map
      Ok(())
    } else {
      Err(Error::ProcessNotFoundError)
    }
  }

  #[allow(dead_code)]
  fn list(&self) -> Vec<Process> {
    self.alloced.clone()
  }

  fn lookup(&self, pid: Pid) -> Option<Process> {
    for i in self.alloced.iter() {
      if i.pid() == pid {
        return Some(i.clone());
      }
    }
    None
  }
}

lazy_static! {
  static ref PROCESS_MAP: Mutex<BTreeMap<Pid, Box<ControlBlock>>> = Mutex::new(BTreeMap::new());
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

pub fn free(p: Process) {
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
  let pool = PROCESS_POOL.lock();
  let r = pool.lookup(pid);
  drop(pool);
  r
}

pub fn create(elf: &'static [u8], arg: usize) {
  let p = alloc(None);
  unsafe {
    let page_table = (*p.pcb()).page_table;
    let pc = super::elf::load_elf(elf, page_table);
    let sp = CONFIG_USER_STACK_TOP;
    match page_table.insert_page(sp - PAGE_SIZE, crate::mm::page_pool::alloc(), EntryAttribute::user_default()) {
      Ok(_) => {}
      Err(_) => { panic!("process: load_image: page_table.insert_page failed") }
    }
    let t = crate::lib::thread::alloc_user(pc, sp, arg, p);
    t.set_status(crate::lib::thread::Status::TsRunnable);
    p.set_main_thread(t);
  }
}