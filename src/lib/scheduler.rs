use alloc::vec::Vec;
use lib::process::Process;

pub struct RoundRobinScheduler {
  counter: usize,
}

pub trait Scheduler {
  fn schedule(&mut self, candidates: Vec<Process>);
}

impl Scheduler for RoundRobinScheduler {
  fn schedule(&mut self, candidates: Vec<Process>) {
    if candidates.is_empty() {
      panic!("scheduler: no runnable process");
    }
    loop {
      self.counter += 1;
      let candidate = self.counter % candidates.len();
      if candidates[candidate].is_runnable() {
        println!("scheduler: switch to [{}]", candidates[candidate].pid());
        candidates[candidate].run();
        return;
      }
    }
  }
}

pub static mut SCHEDULER: RoundRobinScheduler = RoundRobinScheduler { counter: 0 };

pub fn schedule() {
  unsafe {
    SCHEDULER.schedule(crate::lib::process_pool::pid_list());
  }
}