use alloc::vec::Vec;
use core::ops::Range;
use arch::*;
use mm::*;
use spin::Mutex;


struct PageRecord {
  frame: PageFrame,
  free: bool,
  ref_count: usize,
}

struct PagePool {
  pages: Mutex<Vec<PageRecord>>
}

impl PagePool {
  pub fn alloc(&self) -> PageFrame {
    for (i, record) in self.pages.lock().iter_mut().enumerate() {
      if record.free {
        record.free = false;
        return record.frame.clone();
      }
    }
    panic!("PagePool page exhausted");
  }

  pub fn init(&self, range: Range<usize>) {
    if range.start % PAGE_SIZE != 0 {
      panic!("Page frame not align");
    }
    for i in range.step_by(PAGE_SIZE) {
      self.pages.lock().push(PageRecord { frame: PageFrame::new(i), free: true, ref_count: 0 });
    }
  }

  pub fn get_ref_count(&self, frame: PageFrame) -> usize {
    unimplemented!()
  }

  pub fn add_ref_count(&self, frame: PageFrame, incresement: usize) {
    unimplemented!()
  }
}

static PAGE_POOL: PagePool = PagePool { pages: Mutex::new(Vec::new()) };

pub fn init(range: Range<usize>) {
  PAGE_POOL.init(range);
}

pub fn alloc() -> PageFrame {
  PAGE_POOL.alloc()
}
