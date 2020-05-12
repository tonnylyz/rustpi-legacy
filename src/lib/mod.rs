use crate::arch::{Core, CoreTrait};

pub mod print;
pub mod isr;
pub mod process;
pub mod elf;
pub mod user_image;
pub mod scheduler;
pub mod syscall;
pub mod page_table;
pub mod thread;
pub mod bitmap;

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}

pub fn current_core() -> &'static mut Core {
  crate::arch::common::core::current()
}

#[inline(always)]
pub fn current_thread() -> Option<self::thread::Thread> {
  let core = crate::arch::common::core::current();
  core.running_thread()
}

#[inline(always)]
pub fn current_process() -> Option<self::process::Process> {
  match current_thread() {
    None => { None }
    Some(t) => { t.process() }
  }
}
