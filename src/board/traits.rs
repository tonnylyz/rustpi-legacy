use core::ops::Range;

pub trait Board {
  fn physical_address_limit(&self) -> usize;
  fn normal_memory_range(&self) -> Range<usize>;
  fn device_memory_range(&self) -> Range<usize>;
  fn kernel_start(&self) -> usize;
  fn kernel_stack_top(&self) -> usize;
}