use alloc::vec::Vec;
use core::ops::Range;
use arch::*;
use mm::*;
use spin::Mutex;

pub type PageRefCount = u32;
type PagePoolItem = Option<PageRefCount>;

struct PagePool(Mutex<Vec<PagePoolItem>>);

impl PagePool {
  fn pa2index(&self, pa: usize) -> usize {
    if !crate::config::paged_range().contains(&pa) {
      panic!("PagePool pa2index out of index");
    }
    let start = crate::config::paged_range().start;
    (pa - start) / PAGE_SIZE
  }

  fn index2pa(&self, index: usize) -> usize {
    let start = crate::config::paged_range().start;
    index * PAGE_SIZE + start
  }

  pub fn alloc(&self) -> PageFrame {
    let mut vec = self.0.lock();
    for (i, item) in vec.iter_mut().enumerate() {
      if item.is_none() {
        item.replace(0);
        drop(vec);
        return PageFrame::new(self.index2pa(i));
      }
    }
    panic!("PagePool exhausted");
  }

  pub fn free(&self, frame: PageFrame) -> bool {
    let i = self.pa2index(frame.pa());
    let mut vec = self.0.lock();
    if let Some(0) = vec[i] {
      vec[i].take();
      drop(vec);
      return true;
    }
    drop(vec);
    return false;
  }

  pub fn init(&self, range: Range<usize>) {
    let mut vec =self.0.lock();
    for _ in range.step_by(PAGE_SIZE) {
      vec.push(PagePoolItem::None);
    }
    drop(vec);
  }

  pub fn get_ref_count(&self, frame: PageFrame) -> PageRefCount {
    unimplemented!()
  }

  pub fn add_ref_count(&self, frame: PageFrame, incresement: PageRefCount) {
    unimplemented!()
  }
}

static PAGE_POOL: PagePool = PagePool(Mutex::new(Vec::new()));

pub fn init(range: Range<usize>) {
  PAGE_POOL.init(range);
}

pub fn alloc() -> PageFrame {
  PAGE_POOL.alloc()
}

pub fn free(frame: PageFrame) {
  PAGE_POOL.free(frame);
}