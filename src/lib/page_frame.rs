use alloc::vec::Vec;
use core::ops::Range;
use arch::*;

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
    self.pa >> SHIFT_4KB
  }
  pub fn kva(&self) -> usize {
    crate::arch::pa2kva(self.pa)
  }
  pub fn pa(&self) -> usize {
    self.pa
  }
  pub fn zero(&self) {
    for p in (self.kva()..self.kva() + PAGE_SIZE).step_by(8) {
      let p = p as *mut u64;
      unsafe {
        *p = 0;
      }
    }
  }
}

pub fn page_frame_alloc() -> PageFrame {
  unsafe {
    if let Some(frame) = (*PAGE_FRAMES).first() {
      PAGE_FRAMES.remove(0);
      println!("page_frame_alloc alloced {:016x}", frame.pa);
      frame.zero();
      frame.clone()
    } else {
      panic!("Page frame exhausted");
    }
  }
}

static mut PAGE_FRAMES: Vec<PageFrame> = Vec::new();

pub fn init(range: Range<usize>) {
  for i in range.step_by(PAGE_SIZE) {
    unsafe {
      PAGE_FRAMES.push(PageFrame::new(i));
    }
  }
}