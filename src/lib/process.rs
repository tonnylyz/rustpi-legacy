use arch::*;
use config::*;
use lib::scheduler::{SCHEDULER, Scheduler};
use core::fmt::{Display, Formatter, Error};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ProcessStatus {
  PsFree = 0,
  PsRunnable = 1,
  PsNotRunnable = 2,
}

// Process Control Block
#[derive(Copy, Clone, Debug)]
pub struct Process {
  pub id: Option<Pid>,
  pub parent: Option<Pid>,
  pub directory: Option<PageTable>,
  pub context: Option<ContextFrame>,
  pub status: ProcessStatus,

  pub ipc_value: usize,
  pub ipc_from: Option<Pid>,
  pub ipc_receiving: bool,
  pub ipc_dst_addr: usize,
  pub ipc_dst_attr: usize,

  pub exception_handler: usize,
  pub exception_stack_top: usize,
}

impl Display for Process {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    writeln!(f, "Process {}", self.id.unwrap().pid)?;
    writeln!(f, "parent: {:?}", self.parent)?;
    writeln!(f, "directory: {:08x}", self.directory.unwrap().directory().pa())?;
    writeln!(f, "context:\n{}", self.context.unwrap())?;
    writeln!(f ,"status: {:?}", self.status)?;
    writeln!(f ,"ipc_value: {}", self.ipc_value)?;
    writeln!(f ,"ipc_from: {:?}", self.ipc_from)?;
    writeln!(f ,"ipc_receiving: {}", self.ipc_receiving)?;
    writeln!(f ,"ipc_dst_addr: {:016x}", self.ipc_dst_addr)?;
    writeln!(f ,"ipc_dst_attr: {:016x}", self.ipc_dst_attr)?;
    writeln!(f ,"exception_handler: {:016x}", self.exception_handler)?;
    writeln!(f ,"exception_stack_top: {:016x}", self.exception_stack_top)?;
    Ok(())
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Pid {
  pid: u16,
  pcb: usize//*mut Process,
}

impl Pid {
  pub fn new(pid: u16, pcb: *mut Process) -> Self {
    Pid {
      pid,
      pcb: pcb as usize
    }
  }
  
  pub fn pid(&self) -> u16 {
    self.pid
  }
  
  pub fn pcb(&self) -> *mut Process {
    self.pcb as *mut Process
  }
  
  pub fn parent(&self) -> Option<Pid> {
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
      (*self.pcb()).directory = Some(page_table);
      // TODO: map `PROCESS_LIST` to user space
    }
  }

  fn load_image(&self, elf: &'static [u8]) {
    unsafe {
      let page_table = (*self.pcb()).directory.unwrap();
      page_table.insert_page(CONFIG_USER_STACK_TOP - PAGE_SIZE, crate::mm::page_pool::alloc(), PageTableEntryAttr::user_default());
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
      panic!("create alloc error");
    }
  }

  pub fn free(&self) {
    // TODO: traverse whole user address space, recycle all page frame (page table included)
    super::process_pool::free(self.clone());
  }

  pub fn destroy(&self) {
    self.free();
    unsafe {
      if let Some(pid) = CURRENT_PROCESS {
        if pid.pid == self.pid {
          CURRENT_PROCESS = None;
          SCHEDULER.schedule(super::process_pool::pid_list());
        }
      }
    }
  }

  pub fn run(&self) {
    unsafe {
      assert!((*self.pcb()).directory.is_some());
      assert!((*self.pcb()).context.is_some());
      if let Some(prev) = CURRENT_PROCESS {
        (*(prev.pcb())).context = Some(CONTEXT_FRAME);
      }
      CURRENT_PROCESS = Some(self.clone());
      CONTEXT_FRAME = (*self.pcb()).context.unwrap();
      crate::arch::ARCH.set_user_page_table((*self.pcb()).directory.unwrap(), self.pid as AddressSpaceId);
      crate::arch::ARCH.invalidate_tlb();
    }
  }

  pub fn is_runnable(&self) -> bool {
    unsafe {
      (*self.pcb()).status == ProcessStatus::PsRunnable
    }
  }
}

pub static mut CURRENT_PROCESS: Option<Pid> = None;