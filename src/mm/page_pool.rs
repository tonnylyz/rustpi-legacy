use alloc::vec::Vec;
use core::ops::Range;
use arch::*;
use mm::*;
use spin::Mutex;

pub type PageRefCount = u32;
pub type PagePoolItem = Option<PageRefCount>;

struct PagePool {
  start: Option<usize>,
  end: Option<usize>,
  vec: Vec<PagePoolItem>,
}

pub trait PagePoolImpl {
  fn init(&mut self, range: Range<usize>);
  fn alloc(&mut self) -> PageFrame;
  fn free(&mut self, frame: PageFrame) -> Result<(),()>;
}

impl PagePoolImpl for PagePool {

  fn init(&mut self, range: Range<usize>) {
    self.start.replace(range.start);
    self.end.replace(range.end);
    for _ in range.step_by(PAGE_SIZE) {
      self.vec.push(PagePoolItem::None);
    }
  }

  fn alloc(&mut self) -> PageFrame {
    for (i, item) in self.vec.iter_mut().enumerate() {
      if item.is_none() {
        item.replace(0);
        return PageFrame::new(self.start.unwrap() + i * PAGE_SIZE);
      }
    }
    panic!("PagePool exhausted");
  }

  fn free(&mut self, frame: PageFrame) -> Result<(),()> {
    if !self.in_range(frame.pa()) {
      panic!("PagePool free frame not managed");
    }
    let i = (frame.pa() - self.start.unwrap()) / PAGE_SIZE;
    if let Some(0) = self.vec[i] {
      self.vec[i].take();
      Ok(())
    } else {
      Err(())
    }
  }

}

impl PagePool {
  pub const fn new() -> Self {
    PagePool {
      start: None,
      end: None,
      vec: Vec::new(),
    }
  }

  pub fn in_range(&self, pa: usize) -> bool {
    (self.start.unwrap()..self.end.unwrap()).contains(&pa)
  }

  pub fn get_ref_count(&self, frame: PageFrame) -> PagePoolItem {
    if !self.in_range(frame.pa()) {
      panic!("PagePool free frame not managed");
    }
    let i = (frame.pa() - self.start.unwrap()) / PAGE_SIZE;
    self.vec[i].clone()
  }

  pub fn add_ref_count(&mut self, frame: PageFrame, delta: u32) {
    if !self.in_range(frame.pa()) {
      panic!("PagePool free frame not managed");
    }
    let i = (frame.pa() - self.start.unwrap()) / PAGE_SIZE;
    let before = self.vec[i].take().unwrap();
    self.vec[i].replace(before + delta);
  }
}

static PAGE_POOL: Mutex<PagePool> = Mutex::new(PagePool::new());

pub fn init(range: Range<usize>) {
  let mut pool = PAGE_POOL.lock();
  pool.init(range);
  drop(pool);
}

pub fn alloc() -> PageFrame {
  let mut pool = PAGE_POOL.lock();
  let r = pool.alloc();
  drop(pool);
  return r;
}

pub fn free(frame: PageFrame) {
  let mut pool = PAGE_POOL.lock();
  pool.free(frame);
  drop(pool);
}
