use alloc::vec::Vec;
use arch::*;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ProcessStatus {
  Running,
  Ready,
  NotReady,
}

// Process Control Block
#[derive(Copy, Clone)]
struct Process {
  id: u8,
  page_table: Option<PageTable>,
  context: Option<ContextFrame>,
  status: ProcessStatus,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Pid(u8);

impl core::fmt::Display for Pid {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "[PID: {}]", self.0)
  }
}

impl Pid {
  pub fn init(&self, arg: usize, entry_point: usize) {
    let pid = (*self).0;
    unsafe {
      let mut ctx = ContextFrame::default();
      ctx.gpr[0] = arg as u64;
      ctx.elr = entry_point as u64;
      PROCESSES[pid as usize].context = Some(ctx);
    }
  }

  pub fn set_page_table(&self, page_table: PageTable) {
    let pid = (*self).0;
    unsafe {
      PROCESSES[pid as usize].page_table = Some(page_table);
    }
  }

  pub fn get_page_table(&self) -> PageTable {
    let pid = (*self).0;
    unsafe {
      if let Some(page_table) = PROCESSES[pid as usize].page_table {
        page_table
      } else {
        panic!("Page table not set");
      }
    }
  }

  pub fn save_context_to_pcb(&self) {
    let pid = (*self).0;
    unsafe {
      PROCESSES[pid as usize].context = Some(CONTEXT_FRAME);
    }
  }

  pub fn set_status(&self, status: ProcessStatus) {
    let pid = (*self).0;
    unsafe {
      PROCESSES[pid as usize].status = status;
    }
  }

  pub fn sched(&self) {
    let pid = (*self).0;
    unsafe {
      if PROCESSES[pid as usize].page_table.is_none() {
        panic!("Process page table not exists")
      }
      if PROCESSES[pid as usize].context.is_none() {
        panic!("Process context not exists")
      }
      PROCESSES[pid as usize].status = ProcessStatus::Running;
      crate::arch::ARCH.set_user_page_table(PROCESSES[pid as usize].page_table.unwrap(), pid as AddressSpaceId);
      CONTEXT_FRAME = PROCESSES[pid as usize].context.unwrap();
    }
  }
}

pub fn process_alloc() -> Pid {
  unsafe {
    let pid = PROCESSES.len() as u8;
    PROCESSES.push(Process {
      id: pid,
      page_table: None,
      context: None,
      status: ProcessStatus::NotReady,
    });
    Pid(pid)
  }
}

pub fn process_current() -> Option<Pid> {
  unsafe {
    for (_i, p) in PROCESSES.iter().enumerate() {
      if p.status == ProcessStatus::Running {
        return Some(Pid(p.id));
      }
    }
  }
  None
}

pub fn process_next_ready() -> Option<Pid> {
  unsafe {
    for (_i, p) in PROCESSES.iter().enumerate() {
      if p.status == ProcessStatus::Ready {
        return Some(Pid(p.id));
      }
    }
  }
  None
}

pub fn process_schedule() {
  if let Some(next) = process_next_ready() {
    if let Some(current) = process_current() {
      println!("switch from {} to {}", current, next);
      if current == next {
        return;
      }
      current.set_status(ProcessStatus::Ready); // Running -> Ready
      current.save_context_to_pcb();
      next.sched();
    }
  } else {
    return; // no ready process
  }
}

static mut PROCESSES: Vec<Process> = Vec::new();