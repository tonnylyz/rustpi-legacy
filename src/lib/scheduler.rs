use alloc::vec::Vec;

use crate::arch::ArchTrait;
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
    let core_id = crate::arch::Arch::core_id();
    loop {
      //for i in crate::lib::process_pool::pid_list().iter() {
      //  if i.pid() == match core_id { 1 => 1024, 2 => 1023, 3 => 1022, _ => 0 } {
      //    if i.is_runnable() {
      //      println!("\ncore_{} scheduler: switch to [{}]", core_id, i.pid());
      //      i.run();
      //      return;
      //    }
      //  }
      //}

      self.counter += 1;
      let candidates: Vec<Process> = crate::lib::process_pool::pid_list();
      if candidates.is_empty() {
        continue;
      }
      let i = self.counter % candidates.len();
      let candidate = candidates[i];
      if candidates[i].is_runnable() {
        candidate.lock();
        if let Some(_) = candidate.running_at() {
          candidate.unlock();
          continue;
        } else {
          candidate.set_running_at(Some(core_id));
          candidate.unlock();
        }
        println!("\ncore_{} scheduler: switch to [{}]", core_id, candidates[i].pid());
        candidates[i].run();
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
  crate::arch::Arch::schedule();
}