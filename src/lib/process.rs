use arch::*;
use config::*;
use lib::scheduler::{SCHEDULER, Scheduler};

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
      (*self.pcb()).directory = Some(PageTable::new(frame));
      // TODO: map `PROCESS_LIST` to user space
      // TODO: recursive page table
    }
  }

  fn load_image(&self, elf: &'static [u8]) {
    unsafe {
      let page_table = (*self.pcb()).directory.unwrap();
      page_table.insert_page(CONFIG_PROCESS_STACK_TOP - PAGE_SIZE, crate::mm::page_pool::alloc(), PteAttribute::user_default());
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