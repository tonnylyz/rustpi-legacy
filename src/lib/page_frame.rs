use alloc::vec::Vec;
use driver::mmio::mmio_write;

#[derive(Clone, Copy, Debug)]
pub struct PageFrame {
  pa: usize,
  ref_count: usize,
  free: bool,
}

impl PageFrame {
  pub fn new(pa : usize) -> Self {
    PageFrame{
      pa,
      ref_count: 0,
      free: true,
    }
  }
  pub fn ppn(&self) -> usize {
    self.pa >> 12
  }
  pub fn kva(&self) -> usize {
    self.pa | 0xFFFFFF8000000000
  }
  pub fn pa(&self) -> usize {
    self.pa
  }
  pub fn zero(&self) {
    for p in (self.kva()..self.kva() + 4096).step_by(8) {
      let p = p as *mut u64;
      unsafe {
        *p = 0;
      }
    }
  }
}

pub fn page_frame_alloc() -> PageFrame {
  let mut r : PageFrame;
  unsafe {
    for (i, frame) in PAGE_FRAMES.iter().enumerate() {
      if frame.free {
        PAGE_FRAMES.remove(i);
        r = *frame;
        r.ref_count = 0;
        r.zero();
        return r;
      }
    }
  }
  panic!("Page frame exhausted");
}


pub static mut PAGE_FRAMES : Vec<PageFrame> = Vec::new();