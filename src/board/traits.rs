use core::ops::Range;

pub trait BoardTrait {
  fn physical_address_limit() -> usize;
  fn normal_memory_range() -> Range<usize>;
  fn device_memory_range() -> Range<usize>;
  fn kernel_start() -> usize;
  fn kernel_stack_top() -> usize;
  fn kernel_stack_top_core(core_id: usize) -> usize;
}