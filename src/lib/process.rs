use arch::*;
use config::*;
use core::fmt::{Display, Formatter};
use core::intrinsics::size_of;

pub type Pid = u16;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ProcessStatus {
  PsFree = 0,
  PsRunnable = 1,
  PsNotRunnable = 2,
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct InterProcessComm {
  pub id: Pid,
  pub ipc_from: Pid,
  pub ipc_receiving: bool,
  pub ipc_value: usize,
  pub ipc_dst_addr: usize,
  pub ipc_dst_attr: usize,
}

#[no_mangle]
#[link_section = ".bss.ipc"]
pub static mut IPC_LIST: [InterProcessComm; CONFIG_PROCESS_NUMBER] = [InterProcessComm{
  id: 0,
  ipc_from: 0,
  ipc_receiving: false,
  ipc_value: 0,
  ipc_dst_addr: 0,
  ipc_dst_attr: 0
}; CONFIG_PROCESS_NUMBER];

#[derive(Copy, Clone, Debug)]
pub struct ProcessControlBlock {
  pub id: Pid,
  pub parent: Option<Process>,
  pub directory: Option<PageTable>,
  pub context: Option<ContextFrame>,
  pub status: ProcessStatus,

  pub exception_handler: usize,
  pub exception_stack_top: usize,
}

#[no_mangle]
pub static mut PCB_LIST: [ProcessControlBlock; CONFIG_PROCESS_NUMBER] = [ProcessControlBlock {
  id: 0,
  parent: None,
  directory: None,
  context: None,
  status: ProcessStatus::PsFree,
  exception_handler: 0,
  exception_stack_top: 0
}; CONFIG_PROCESS_NUMBER];


impl Display for ProcessControlBlock {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    writeln!(f, "Process {}", self.id)?;
    writeln!(f, "parent: {:?}", self.parent)?;
    writeln!(f, "directory: {:08x}", self.directory.unwrap().directory().pa())?;
    writeln!(f, "context:\n{}", self.context.unwrap())?;
    writeln!(f ,"status: {:?}", self.status)?;
    writeln!(f ,"exception_handler: {:016x}", self.exception_handler)?;
    writeln!(f ,"exception_stack_top: {:016x}", self.exception_stack_top)?;
    Ok(())
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Process {
  pid: Pid
}

impl Process {
  pub fn new(pid: Pid) -> Self {
    assert_ne!(pid, 0);
    Process {
      pid,
    }
  }
  
  pub fn pid(&self) -> Pid {
    self.pid
  }

  pub fn index(&self) -> usize {
    (self.pid - 1) as usize
  }

  pub fn pcb(&self) -> *mut ProcessControlBlock {
    unsafe {
      (&mut PCB_LIST[self.index()]) as *mut ProcessControlBlock
    }
  }

  pub fn ipc(&self) -> *mut InterProcessComm {
    unsafe {
      (&mut IPC_LIST[self.index()]) as *mut InterProcessComm
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
      assert_eq!(size_of::<InterProcessComm>(), CONFIG_PROCESS_IPC_SIZE);
      for i in 0..(CONFIG_PROCESS_IPC_SIZE * CONFIG_PROCESS_NUMBER / PAGE_SIZE) {
        let va = CONFIG_USER_IPC_LIST_BTM + i * PAGE_SIZE;
        let pa = kva2pa(&IPC_LIST[i * (PAGE_SIZE / CONFIG_PROCESS_IPC_SIZE)] as *const InterProcessComm as usize);
        page_table.map(va, pa, PageTableEntryAttr::readonly());
      }
      (*self.pcb()).directory = Some(page_table);
    }
  }

  fn load_image(&self, elf: &'static [u8]) {
    unsafe {
      let page_table = (*self.pcb()).directory.unwrap();
      match page_table.insert_page(CONFIG_USER_STACK_TOP - PAGE_SIZE, crate::mm::page_pool::alloc(), PageTableEntryAttr::user_default()) {
        Ok(_) => {},
        Err(_) => { panic!("process: load_image page_table.insert_page failed") },
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
      panic!("create alloc error");
    }
  }

  pub fn free(&self) {
    unsafe {
      (*self.pcb()).directory.unwrap().destroy();
      let frame = (*self.pcb()).directory.unwrap().directory();
      crate::mm::page_pool::decrease_rc(frame);
    }
    super::process_pool::free(self.clone());
  }

  pub fn destroy(&self) {
    self.free();
    unsafe {
      if let Some(pid) = CURRENT_PROCESS {
        if pid.pid == self.pid {
          CURRENT_PROCESS = None;
          crate::lib::scheduler::schedule();
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

pub static mut CURRENT_PROCESS: Option<Process> = None;