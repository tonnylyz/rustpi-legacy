use core::fmt::{Display, Formatter};

use arch::*;
use config::*;
use lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};

pub type Pid = u16;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Status {
  PsFree = 0,
  PsRunnable = 1,
  PsNotRunnable = 2,
}

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
  pub id: Pid,
  pub parent: Option<Process>,
  pub page_table: Option<PageTable>,
  pub context: Option<ContextFrame>,
  pub status: Status,
  pub exception_handler: usize,
  pub exception_stack_top: usize,

  lock: spin::Mutex<()>,
  running_at: Option<usize>, // core_id
}

#[no_mangle]
pub static mut PCB_LIST: [ControlBlock; CONFIG_PROCESS_NUMBER] = [ControlBlock {
  id: 0,
  parent: None,
  page_table: None,
  context: None,
  status: Status::PsFree,
  exception_handler: 0,
  exception_stack_top: 0,
  lock: spin::Mutex::new(()),
  running_at: None
}; CONFIG_PROCESS_NUMBER];


impl Display for ControlBlock {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    writeln!(f, "Process {}", self.id)?;
    writeln!(f, "parent: {:?}", self.parent)?;
    writeln!(f, "directory: {:08x}", self.page_table.unwrap().directory().pa())?;
    writeln!(f, "context:\n{}", self.context.unwrap())?;
    writeln!(f, "status: {:?}", self.status)?;
    writeln!(f, "exception_handler: {:016x}", self.exception_handler)?;
    writeln!(f, "exception_stack_top: {:016x}", self.exception_stack_top)?;
    Ok(())
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Process {
  pid: Pid,
}

impl Process {
  pub fn new(pid: Pid) -> Self {
    assert_ne!(pid, 0);
    Process {
      pid
    }
  }

  pub fn pid(&self) -> Pid {
    self.pid
  }

  pub fn index(&self) -> usize {
    (self.pid - 1) as usize
  }

  pub fn pcb(&self) -> *mut ControlBlock {
    unsafe {
      (&mut PCB_LIST[self.index()]) as *mut ControlBlock
    }
  }

  pub fn ipc(&self) -> *mut Ipc {
    unsafe {
      (&mut IPC_LIST[self.index()]) as *mut Ipc
    }
  }

  pub fn parent(&self) -> Option<Process> {
    unsafe {
      (*self.pcb()).parent
    }
  }

  pub fn setup_vm(&self) {
    unsafe {
      let frame = crate::mm::page_pool::alloc();
      crate::mm::page_pool::increase_rc(frame);
      let page_table = PageTable::new(frame);
      page_table.recursive_map(CONFIG_RECURSIVE_PAGE_TABLE_BTM);
      for i in 0..(CONFIG_PROCESS_IPC_SIZE * CONFIG_PROCESS_NUMBER / PAGE_SIZE) {
        let va = CONFIG_USER_IPC_LIST_BTM + i * PAGE_SIZE;
        let pa = kva2pa(&IPC_LIST[i * (PAGE_SIZE / CONFIG_PROCESS_IPC_SIZE)] as *const Ipc as usize);
        page_table.map(va, pa, EntryAttribute::user_readonly());
      }
      (*self.pcb()).page_table = Some(page_table);
    }
  }

  fn load_image(&self, elf: &'static [u8]) {
    unsafe {
      let page_table = (*self.pcb()).page_table.unwrap();
      match page_table.insert_page(CONFIG_USER_STACK_TOP - PAGE_SIZE, crate::mm::page_pool::alloc(), EntryAttribute::user_default()) {
        Ok(_) => {}
        Err(_) => { panic!("process: load_image: page_table.insert_page failed") }
      }
      let entry = super::elf::load_elf(elf, page_table);
      let mut ctx = (*self.pcb()).context.unwrap();
      ctx.set_exception_pc(entry);
      (*self.pcb()).context = Some(ctx);
    }
  }

  pub fn create(elf: &'static [u8], arg: usize) {
    if let Ok(pid) = super::process_pool::alloc(None, arg) {
      pid.load_image(elf);
    } else {
      panic!("process: create: alloc error");
    }
  }

  pub fn free(&self) {
    self.unlock();
    unsafe {
      (*self.pcb()).page_table.unwrap().destroy();
      let frame = (*self.pcb()).page_table.unwrap().directory();
      crate::mm::page_pool::decrease_rc(frame);
    }
    super::process_pool::free(self.clone());
  }

  pub fn destroy(&self) {
    self.free();
    if let Some(pid) = crate::arch::Arch::running_process() {
      if pid.pid == self.pid {
        crate::arch::Arch::set_running_process(None);
        crate::lib::scheduler::schedule();
      }
    }
  }

  pub fn run(&self) {
    unsafe {
      assert!((*self.pcb()).page_table.is_some());
      assert!((*self.pcb()).context.is_some());
      if let Some(prev) = crate::arch::Arch::running_process() {
        // Note: normal switch
        prev.lock();
        prev.set_running_at(None);
        (*(prev.pcb())).context = Some(*crate::arch::Arch::context());
        prev.unlock();
        (*crate::arch::Arch::context()) = (*self.pcb()).context.unwrap();
      } else if crate::arch::Arch::has_context() {
        // Note: previous process has been destroyed
        (*crate::arch::Arch::context()) = (*self.pcb()).context.unwrap();
      } else {
        // Note: this is first run
        // Arch::start_first_process prepare the context to stack
      }
      crate::arch::Arch::set_running_process(Some(self.clone()));
      crate::arch::Arch::set_user_page_table((*self.pcb()).page_table.unwrap(), self.pid as AddressSpaceId);
      crate::arch::Arch::invalidate_tlb();
    }
  }

  pub fn is_runnable(&self) -> bool {
    unsafe {
      (*self.pcb()).status == Status::PsRunnable
    }
  }

  pub fn lock(&self) {
    unsafe {
      (*self.pcb()).lock.lock();
    }
  }

  pub fn unlock(&self) {
    unsafe {
      (*self.pcb()).lock.force_unlock()
    }
  }

  pub fn running_at(&self) -> Option<usize> {
    unsafe {
      (*self.pcb()).running_at
    }
  }

  pub fn set_running_at(&self, core: Option<usize>) {
    unsafe {
      (*self.pcb()).running_at = core;
    }
  }
}
