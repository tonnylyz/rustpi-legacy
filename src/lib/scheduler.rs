use alloc::vec::Vec;
use lib::process::Pid;

pub struct RoundRobinScheduler {
  counter: usize,
}

pub trait Scheduler {
  fn schedule(&mut self, candidates: Vec<Pid>);
}

impl Scheduler for RoundRobinScheduler {
  fn schedule(&mut self, candidates: Vec<Pid>) {
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
