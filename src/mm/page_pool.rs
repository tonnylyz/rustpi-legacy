use alloc::vec::Vec;
use core::ops::Range;

use spin::Mutex;

use crate::arch::*;
use crate::mm::PageFrame;

use self::Error::*;

#[derive(Copy, Clone, Debug)]
pub enum Error {
  OutOfFrameError,
  UnmanagedFrameError,
  FreeUnallocatedFrameError,
  FreeReferencedFrameError,
  RefCountOverflowError,
}

struct PagePool {
  start: usize,
  end: usize,
  rc: Vec<u8>,
  free: Vec<usize>,
  allocated: Vec<usize>,
}

pub trait PagePoolTrait {
  fn init(&mut self, range: Range<usize>);
  fn allocate(&mut self) -> Result<PageFrame, Error>;
  fn free(&mut self, frame: PageFrame) -> Result<(), Error>;
  fn increase_rc(&mut self, frame: PageFrame) -> Result<u8, Error>;
  fn decrease_rc(&mut self, frame: PageFrame) -> Result<u8, Error>;
  fn rc(&self, frame: PageFrame) -> Result<u8, Error>;
  fn ppn(&self, frame: PageFrame) -> usize;
  fn in_pool(&self, frame: PageFrame) -> bool;
  fn report(&self);
}

impl PagePoolTrait for PagePool {
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

  fn allocate(&mut self) -> Result<PageFrame, Error> {
    if let Some(pa) = self.free.pop() {
      self.allocated.push(pa);
      Ok(PageFrame::new(pa))
    } else {
      Err(OutOfFrameError)
    }
  }

  fn free(&mut self, frame: PageFrame) -> Result<(), Error> {
    if !self.in_pool(frame) {
      return Err(UnmanagedFrameError);
    }
    if let Ok(0) = self.rc(frame) {
      if let Some(pa) = self.allocated.remove_item(&frame.pa()) {
        self.free.push(pa);
        Ok(())
      } else {
        Err(FreeUnallocatedFrameError)
      }
    } else {
      Err(FreeReferencedFrameError)
    }
  }

  fn increase_rc(&mut self, frame: PageFrame) -> Result<u8, Error> {
    if !self.in_pool(frame) {
      return Err(UnmanagedFrameError);
    }
    let ppn = self.ppn(frame);
    let val = self.rc[ppn];
    if val == 255 {
      return Err(RefCountOverflowError);
    }
    self.rc[ppn] += 1;
    Ok(val + 1)
  }

  fn decrease_rc(&mut self, frame: PageFrame) -> Result<u8, Error> {
    if !self.in_pool(frame) {
      return Err(UnmanagedFrameError);
    }
    let ppn = self.ppn(frame);
    self.rc[ppn] -= 1;
    if self.rc[ppn] == 0 {
      self.free(frame)?;
      return Ok(0);
    }
    Ok(self.rc[ppn])
  }

  fn rc(&self, frame: PageFrame) -> Result<u8, Error> {
    if !self.in_pool(frame) {
      Err(UnmanagedFrameError)
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

  fn report(&self) {
    println!("page_pool report");
    println!("free:      0x{:08x}", self.free.len());
    println!("allocated: 0x{:08x}", self.allocated.len());
  }
}

impl PagePool {
  const fn new() -> Self {
    PagePool {
      start: 0,
      end: 0,
      rc: Vec::new(),
      free: Vec::new(),
      allocated: Vec::new(),
    }
  }
}

static PAGE_POOL: Mutex<PagePool> = Mutex::new(PagePool::new());

pub fn init() {
  let range = super::config::paged_range();
  let mut pool = PAGE_POOL.lock();
  pool.init(range);
  drop(pool);
}

pub fn alloc() -> PageFrame {
  let mut pool = PAGE_POOL.lock();
  if let Ok(frame) = pool.allocate() {
    drop(pool);
    frame
  } else {
    panic!("page_pool: alloc failed")
  }
}

pub fn try_alloc() -> Result<PageFrame, Error> {
  let mut pool = PAGE_POOL.lock();
  let r = pool.allocate();
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

#[allow(dead_code)]
pub fn report() {
  let pool = PAGE_POOL.lock();
  pool.report();
  drop(pool);
}