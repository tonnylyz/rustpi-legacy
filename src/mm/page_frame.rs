use arch::{PAGE_SHIFT, PAGE_SIZE};

#[derive(Clone, Copy, Debug)]
pub struct PageFrame {
  pa: usize,
}

impl PageFrame {
  pub fn new(pa: usize) -> Self {
    PageFrame {
      pa,
    }
  }
  pub fn ppn(&self) -> usize {
    self.pa >> PAGE_SHIFT
  }
  pub fn kva(&self) -> usize {
    crate::arch::pa2kva(self.pa)
  }
  pub fn pa(&self) -> usize {
    self.pa
  }
  pub fn zero(&self) {
    unsafe {
      core::intrinsics::volatile_set_memory(self.kva() as *mut u8, 0, PAGE_SIZE);
    }
  }
  pub fn copy_to(&self, dest: &PageFrame) {
    unsafe {
      core::intrinsics::volatile_copy_memory(dest.kva() as *mut u8, self.kva() as *mut u8, PAGE_SIZE);
    }
  }
  pub fn copy_from(&self, src: &PageFrame) {
    unsafe {
      core::intrinsics::volatile_copy_memory(self.kva() as *mut u8, src.kva() as *mut u8, PAGE_SIZE);
    }
  }
}
