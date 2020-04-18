use alloc::vec::Vec;

use crate::arch::{ArchTrait, CoreTrait};
use crate::lib::process::Process;

#[derive(Copy, Clone)]
pub struct RoundRobinScheduler {
  counter: usize,
}

pub trait SchedulerTrait {
  fn schedule(&mut self);
}

impl SchedulerTrait for RoundRobinScheduler {
  fn schedule(&mut self) {
    loop {
      self.counter += 1;
      let candidates: Vec<Process> = crate::lib::process_pool::pid_list();
      if candidates.is_empty() {
        crate::arch::Arch::wait_for_event();
        continue;
      }
      let i = self.counter % candidates.len();
      let p = candidates[i];
      if p.is_runnable() {
        println!("\nscheduler: switch to [{}]", p.pid());
        p.run();
        return;
      }
    }
  }
}

impl RoundRobinScheduler {
  pub const fn new() -> Self {
    RoundRobinScheduler {
      counter: 0
    }
  }
}

pub fn schedule() {
  unsafe {
    (*crate::arch::Core::current()).schedule();
  }
}