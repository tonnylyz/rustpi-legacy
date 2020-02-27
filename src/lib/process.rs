use lib::uvm::UserPageTable;
use lib::exception::TrapFrame;
use core::borrow::Borrow;

const PROCESS_NUM_MAX: usize = 128;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ProcessStatus {
  Running,
  Ready,
  Allocated,
  Free,
}

// Process Control Block
#[derive(Copy, Clone)]
struct Process {
  id: u8,
  page_table: Option<UserPageTable>,
  context: Option<TrapFrame>,
  entry: u64,
  status: ProcessStatus,
}

global_asm!(include_str!("program.S"));

pub fn process_init() {
  unsafe {
    for (i, pcb) in PROCESSES.iter_mut().enumerate() {
      pcb.id = (i + 1) as u8;
      pcb.status = ProcessStatus::Free;
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Pid(u8);

impl Pid {
  pub fn init_context(&self) {
    let pid = (*self).0;
    unsafe {
      PROCESSES[pid as usize].context = Some(TrapFrame::default());
    }
  }

  pub fn set_page_table(&self, upt: UserPageTable) {
    let pid = (*self).0;
    unsafe {
      PROCESSES[pid as usize].page_table = Some(upt);
    }
  }

  pub fn get_page_table(&self) -> UserPageTable {
    let pid = (*self).0;
    unsafe {
      if let Some(upt) = PROCESSES[pid as usize].page_table {
        upt
      } else {
        panic!("Page table not set");
      }
    }
  }

  pub fn sched(&self) -> ! {
    let pid = (*self).0;
    extern {
      fn pop_time_stack() -> !;
    }
    unsafe {
      if PROCESSES[pid as usize].page_table.is_none() {
        panic!("Process page table not exists")
      }
      if PROCESSES[pid as usize].context.is_none() {
        panic!("Process context not exists")
      }
      PROCESSES[pid as usize].page_table.unwrap().install(pid as u16);
      super::exception::TRAP_FRAME = PROCESSES[pid as usize].context.unwrap();
      pop_time_stack();
    }
  }
}

pub fn process_alloc() -> Pid {
  unsafe {
    for (i, pcb) in PROCESSES.iter_mut().enumerate() {
      if pcb.status == ProcessStatus::Free {
        pcb.status = ProcessStatus::Allocated;
        return Pid(i as u8);
      }
    }
  }
  panic!("PCB exhausted");
}

static mut PROCESSES: [Process; PROCESS_NUM_MAX] = [Process {
  id: 0,
  page_table: None,
  context: None,
  entry: 0,
  status: ProcessStatus::Free,
}; PROCESS_NUM_MAX];
