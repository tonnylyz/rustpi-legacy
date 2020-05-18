use crate::arch::CoreTrait;
use crate::lib::current_core;

#[derive(Copy, Clone)]
pub struct RoundRobinScheduler {
  counter: usize,
}

pub trait SchedulerTrait {
  fn schedule(&mut self);
}

impl SchedulerTrait for RoundRobinScheduler {
  fn schedule(&mut self) {
    self.counter += 1;
    let list = crate::lib::thread::list();
    for i in (self.counter % list.len())..list.len() {
      let t = list[i].clone();
      if t.runnable() {
        if t.run() {
          return;
        }
      }
    }
    for i in 0..list.len() {
      let t = list[i].clone();
      if t.runnable() {
        if t.run() {
          return;
        }
      }
    }
    // if no other runnable thread, run idle thread
    let core = current_core();
    let t = core.idle_thread();
    assert!(t.run());
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