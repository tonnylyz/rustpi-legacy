use alloc::vec::Vec;
use core::ops::Range;
use arch::*;
use mm::*;
use spin::Mutex;

#[derive(Copy, Clone, Debug)]
pub enum PagePoolError {
  PagePoolExhausted,
  PageFrameNotManaged,
  PageFrameNotAlloced,
  PageFrameRefCountNotZero,
  PageRefCountOverflow,
}
use self::PagePoolError::*;

struct PagePool {
  start: usize,
  end: usize,
  rc: Vec<u8>,
  free: Vec<usize>,
  alloced: Vec<usize>,
}

pub trait PagePoolImpl {
  fn init(&mut self, range: Range<usize>);
  fn alloc(&mut self) -> Result<PageFrame, PagePoolError>;
  fn free(&mut self, frame: PageFrame) -> Result<(), PagePoolError>;
  fn increase_rc(&mut self, frame: PageFrame) -> Result<u8, PagePoolError>;
  fn decrease_rc(&mut self, frame: PageFrame) -> Result<u8, PagePoolError>;
  fn get_rc(&self, frame: PageFrame) -> Result<u8, PagePoolError>;

  fn ppn(&self, frame: PageFrame) -> usize;
  fn in_pool(&self, frame: PageFrame) -> bool;
}

impl PagePoolImpl for PagePool {
  fn init(&mut self, range: Range<usize>) {
    assert_eq!(range.start % PAGE_SIZE, 0);
    assert_eq!(range.end % PAGE_SIZE, 0);
    self.start = range.start;
    self.end = range.end;
    for pa in range.step_by(PAGE_SIZE) {
      self.rc.push(0);
      self.free.push(pa);
    }
  }

  fn alloc(&mut self) -> Result<PageFrame, PagePoolError> {
    if let Some(pa) = self.free.pop() {
      self.alloced.push(pa);
      Ok(PageFrame::new(pa))
    } else {
      Err(PagePoolExhausted)
    }
  }

  fn free(&mut self, frame: PageFrame) -> Result<(), PagePoolError> {
    if !self.in_pool(frame) {
      return Err(PageFrameNotManaged);
    }
    if let Ok(0) = self.get_rc(frame) {
      if let Some(pa) = self.alloced.remove_item(&frame.pa()) {
        self.free.push(pa);
        Ok(())
      } else {
        Err(PageFrameNotAlloced)
      }
    } else {
      Err(PageFrameRefCountNotZero)
    }
  }

  fn increase_rc(&mut self, frame: PageFrame) -> Result<u8, PagePoolError> {
    if !self.in_pool(frame) {
      return Err(PageFrameNotManaged);
    }
    let ppn = self.ppn(frame);
    let val = self.rc[ppn];
    if val == 255 {
      return Err(PageRefCountOverflow);
    }
    self.rc[ppn] += 1;
    Ok(val + 1)
  }

  fn decrease_rc(&mut self, frame: PageFrame) -> Result<u8, PagePoolError> {
    if !self.in_pool(frame) {
      return Err(PageFrameNotManaged);
    }
    let ppn = self.ppn(frame);
    let val = self.rc[ppn];
    if val == 0 {
      self.free(frame);
      return Ok(0);
    }
    self.rc[ppn] -= 1;
    Ok(val - 1)
  }

  fn get_rc(&self, frame: PageFrame) -> Result<u8, PagePoolError> {
    if !self.in_pool(frame) {
      Err(PageFrameNotManaged)
    } else {
      Ok(self.rc[self.ppn(frame)])
    }
  }

  fn ppn(&self, frame: PageFrame) -> usize {
    assert!(self.in_pool(frame));
    (frame.pa() - self.start) / PAGE_SIZE
  }

  fn in_pool(&self, frame: PageFrame) -> bool {
    self.start <= frame.pa() && frame.pa() < self.end
  }
}

impl PagePool {
  const fn new() -> Self {
    PagePool {
      start: 0,
      end: 0,
      rc: Vec::new(),
      free: Vec::new(),
      alloced: Vec::new(),
    }
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
  if let Ok(frame) = pool.alloc() {
    drop(pool);
    println!("page pool alloc {:08x}", frame.pa());
    frame
  } else {
    panic!("page_pool alloc failed")
  }
}

pub fn free(frame: PageFrame) -> bool {
  let mut pool = PAGE_POOL.lock();
  let r = pool.free(frame).is_ok();
  drop(pool);
  r
}

pub fn increase_rc(frame: PageFrame) {
  let mut pool = PAGE_POOL.lock();
  let _r = pool.increase_rc(frame);
  drop(pool);
}

pub fn decrease_rc(frame: PageFrame) {
  let mut pool = PAGE_POOL.lock();
  let _r = pool.decrease_rc(frame);
  drop(pool);
}