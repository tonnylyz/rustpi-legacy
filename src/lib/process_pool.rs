use alloc::vec::Vec;
use super::process::*;
use config::*;
use spin::Mutex;

struct ProcessPool {
  next_pid: u16,
  free: Vec<usize>, // Free
  alloced: Vec<Pid>, // Runnable and NotRunnable
}

// Note: PROCESS_LIST is inevitably `unsafe`
// It is also mapped to user space
static mut PROCESS_LIST: [Process; CONFIG_PROCESS_NUMBER] = [Process {
  id: None,
  parent: None,
  directory: None,
  context: None,
  status: ProcessStatus::PsFree,
  ipc_value: 0,
  ipc_from: None,
  ipc_receiving: false,
  ipc_dst_addr: 0,
  ipc_dst_attr: 0,
  exception_handler: 0,
  exception_stack_top: 0
}; CONFIG_PROCESS_NUMBER];

pub enum ProcessPoolError {
  ProcessPoolExhausted,
}

impl ProcessPool {
  fn init(&mut self) {
    self.next_pid = 0;
    unsafe {
      for p in PROCESS_LIST.iter_mut() {
        let ptr = p as *mut Process;
        self.free.push(ptr as usize);
      }
    }
  }

  fn alloc(&mut self, parent: Option<Pid>, arg: usize) -> Result<Pid, ProcessPoolError> {
    unsafe {
      if let Some(pcb) = self.free.pop() {
        let id = self.next_pid;
        self.next_pid += 1;
        let pid = Pid::new(id, pcb as *mut Process);
        let pcb = pcb as *mut Process;
        pid.setup_vm();
        (*pcb).id = Some(pid);
        (*pcb).parent = parent;
        (*pcb).status = ProcessStatus::PsRunnable;
        use arch::{ContextFrame, ContextFrameImpl};
        let mut ctx = ContextFrame::default();
        ctx.set_argument(arg);
        ctx.set_stack_pointer(CONFIG_PROCESS_STACK_TOP);
        (*pcb).context = Some(ctx);
        self.alloced.push(pid);
        Ok(pid)
      } else {
        Err(ProcessPoolError::ProcessPoolExhausted)
      }
    }
  }

  fn free(&mut self, pid: Pid) {
    unimplemented!()
  }

  fn pid_list(&self) -> Vec<Pid> {
    self.alloced.clone()
  }
}

static PROCESS_POOL: Mutex<ProcessPool> = Mutex::new(ProcessPool {
  next_pid: 0,
  free: Vec::new(),
  alloced: Vec::new(),
});

pub fn init() {
  let mut pool = PROCESS_POOL.lock();
  pool.init();
  drop(pool);
}

pub fn alloc(parent: Option<Pid>, arg: usize) -> Result<Pid, ProcessPoolError> {
  let mut pool = PROCESS_POOL.lock();
  let r = pool.alloc(parent, arg);
  drop(pool);
  r
}

pub fn free(pid: Pid) {
  let mut pool = PROCESS_POOL.lock();
  pool.free(pid);
  drop(pool);
}

pub fn pid_list() -> Vec<Pid> {
  let pool = PROCESS_POOL.lock();
  let r = pool.pid_list();
  drop(pool);
  r
}