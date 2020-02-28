pub trait ContextFrameImpl {
  fn default() -> Self;
  fn system_call_argument(&self, i: usize) -> usize;
  fn system_call_number(&self) -> usize;
  fn system_call_set_return_value(&mut self, v: usize);
}

pub trait Arch {
  fn exception_init(&self);

  // Note: kernel runs at privileged mode
  // need to trigger a half process switching
  // Require: a process has been schedule, its
  // context filled in CONTEXT_FRAME, and its
  // page table installed at low address space.
  fn start_first_process(&self) -> !;
}