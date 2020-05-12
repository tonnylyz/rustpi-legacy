use crate::arch::{ArchTrait, ContextFrame, CoreTrait};
use crate::board::BOARD_CORE_NUMBER;
use crate::lib::scheduler::{RoundRobinScheduler, SchedulerTrait};
use crate::lib::thread::Thread;

#[derive(Clone)]
pub struct Core {
  context: Option<*mut ContextFrame>,
  running_thread: Option<Thread>,
  scheduler: RoundRobinScheduler,
}

static mut CORES: [Core; BOARD_CORE_NUMBER] = [Core {
  context: None,
  running_thread: None,
  scheduler: RoundRobinScheduler::new(),
}; BOARD_CORE_NUMBER];

impl CoreTrait for Core {
  fn context(&self) -> ContextFrame {
    unsafe {
      *self.context.unwrap()
    }
  }

  fn context_mut(&self) -> &mut ContextFrame {
    unsafe {
      self.context.unwrap().as_mut().unwrap()
    }
  }

  fn set_context(&mut self, ctx: *mut ContextFrame) {
    self.context = Some(ctx);
  }

  fn clear_context(&mut self) {
    self.context = None;
  }

  fn has_context(&self) -> bool {
    self.context.is_some()
  }

  fn install_context(&self, ctx: &ContextFrame) {
    unsafe {
      *(self.context.unwrap()) = *ctx;
    }
  }

  fn running_thread(&self) -> Option<Thread> {
    self.running_thread
  }

  fn set_running_thread(&mut self, t: Option<Thread>) {
    self.running_thread = t
  }

  fn schedule(&mut self) {
    self.scheduler.schedule();
  }
}

pub fn current() -> &'static mut Core {
  let core_id = crate::arch::Arch::core_id();
  unsafe {
    (&mut CORES[core_id] as *mut Core).as_mut().unwrap()
  }
}
