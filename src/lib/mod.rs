pub mod print;
pub mod isr;
pub mod process;
pub mod process_pool;
pub mod elf;
pub mod user_image;
pub mod scheduler;
pub mod syscall;
pub mod page_table;

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}