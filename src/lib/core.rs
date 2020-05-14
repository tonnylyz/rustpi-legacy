use crate::arch::{ArchTrait, ContextFrame, CoreTrait};
use crate::board::BOARD_CORE_NUMBER;
use crate::lib::scheduler::{RoundRobinScheduler, SchedulerTrait};
use crate::lib::thread::Thread;
use spin::Mutex;

pub struct Core {
  context: Mutex<*mut ContextFrame>,
  running_thread: Mutex<Option<Thread>>,
  scheduler: Mutex<RoundRobinScheduler>,
}

// Note: only the core itself can be allowed to access its `Core`
unsafe impl core::marker::Send for Core {}
unsafe impl core::marker::Sync for Core {}

static CORES: [Core; BOARD_CORE_NUMBER] = [Core {
  context: Mutex::new(0usize as *mut ContextFrame),
  running_thread: Mutex::new(None),
  scheduler: Mutex::new(RoundRobinScheduler::new()),
}; BOARD_CORE_NUMBER];

impl CoreTrait for Core {
  fn context(&self) -> &ContextFrame {
    let lock = self.context.lock();
    let r = unsafe { (*lock).as_ref() }.unwrap();
    drop(lock);
    r
  }

  fn context_mut(&self) -> &mut ContextFrame {
    let lock = self.context.lock();
    let r = unsafe { (*lock).as_mut() }.unwrap();
    drop(lock);
    r
  }

  fn set_context(&self, ctx: *mut ContextFrame) {
    let mut lock = self.context.lock();
    *lock = ctx;
    drop(lock);
  }

  fn clear_context(&self) {
    let mut lock = self.context.lock();
    *lock = 0usize as *mut ContextFrame;
    drop(lock);
  }

  fn has_context(&self) -> bool {
    let lock = self.context.lock();
    let r = unsafe { (*lock).as_ref() }.is_some();
    drop(lock);
    r
  }

  fn running_thread(&self) -> Option<Thread> {
    let lock = self.running_thread.lock();
    let r = lock.clone();
    drop(lock);
    r
  }

  fn set_running_thread(&self, t: Option<Thread>) {
    let mut lock = self.running_thread.lock();
    *lock = t;
    drop(lock);
  }

  fn schedule(&self) {
    let mut lock = self.scheduler.lock();
    lock.schedule();
    drop(lock);
  }
}

pub fn current() -> &'static Core {
  let core_id = crate::arch::Arch::core_id();
  &CORES[core_id]
}
