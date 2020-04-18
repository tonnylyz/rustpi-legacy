use crate::{
  arch::{ArchTrait, ContextFrame, CoreTrait},
  board::BOARD_CORE_NUMBER,
  lib::process::Process,
  lib::scheduler::{RoundRobinScheduler, SchedulerTrait},
};

#[derive(Clone)]
pub struct Core {
  context: Option<*mut ContextFrame>,
  running_process: Option<Process>,
  scheduler: RoundRobinScheduler,
}

static mut CORES: [Core; BOARD_CORE_NUMBER] = [Core {
  context: None,
  running_process: None,
  scheduler: RoundRobinScheduler::new(),
}; BOARD_CORE_NUMBER];

impl CoreTrait for Core {
  fn current() -> *mut Self {
    unsafe {
      &mut CORES[crate::arch::Arch::core_id()]
        as *mut Self
    }
  }

  fn context(&self) -> Option<*mut ContextFrame> {
    self.context
  }

  fn set_context(&mut self, ctx: Option<*mut ContextFrame>) {
    self.context = ctx;
  }

  fn install_context(&self, ctx: ContextFrame) {
    unsafe {
      *(self.context.unwrap()) = ctx;
    }
  }

  fn running_process(&self) -> Option<Process> {
    self.running_process
  }

  fn set_running_process(&mut self, p: Option<Process>) {
    self.running_process = p
  }

  fn schedule(&mut self) {
    self.scheduler.schedule();
  }

  fn start_first_process(&self) -> ! {
    extern {
      fn pop_context_first(ctx: usize) -> !;
    }
    unsafe {
      let p = self.running_process.unwrap();
      let ctx = (*p.pcb()).context.unwrap();
      pop_context_first(&ctx as *const _ as usize);
    }
  }
}