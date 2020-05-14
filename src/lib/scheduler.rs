use alloc::vec::Vec;

use crate::arch::{ArchTrait, CoreTrait};
use crate::lib::current_core;
use crate::lib::thread::Thread;

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
      let candidates: Vec<Thread> = crate::lib::thread::list();
      if candidates.is_empty() {
        crate::arch::Arch::wait_for_event();
        continue;
      }
      let i = self.counter % candidates.len();
      let t = candidates[i].clone();
      if t.runnable() {
        println!("\nscheduler: switch to [{}]", t.tid());
        t.run();
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
  current_core().schedule();
}