pub trait ContextFrameImpl {
  fn default() -> Self;
  fn system_call_argument(&self, i: usize) -> usize;
  fn system_call_number(&self) -> usize;
  fn system_call_set_return_value(&mut self, v: usize);
}

use crate::mm::PageFrame;
use arch::PageTable;

#[derive(Copy, Clone)]
pub struct PtePermission {
  pub r: bool, pub w: bool, pub x: bool
}

#[derive(Copy, Clone)]
pub struct PteAttribute {
  pub cap_kernel: PtePermission,
  pub cap_user: PtePermission,
  pub cow: bool,
  // copy on write
  pub library: bool,
  // share when fork
  pub device: bool,
}

#[derive(Copy, Clone)]
pub struct PageTableEntry {
  pub attr: PteAttribute,
  pub addr: usize,
}

impl PteAttribute {
  pub fn default() -> Self {
    PteAttribute {
      cap_kernel: PtePermission {r: true, w: true, x: true},
      cap_user: PtePermission {r: true, w: true, x: true},
      cow: false,
      library: false,
      device: false
    }
  }
}

pub trait PageTableImpl {
  fn new(directory: PageFrame) -> Self;
  fn install(&self, pid: u16);
  fn map(&self, va: usize, pa: usize, attr: PteAttribute);
  fn map_frame(&self, va: usize, frame: PageFrame, attr: PteAttribute);
  fn unmap(&self, va: usize);
}

pub trait Arch {
  fn exception_init(&self);

  // Note: kernel runs at privileged mode
  // need to trigger a half process switching
  // Require: a process has been schedule, its
  // context filled in CONTEXT_FRAME, and its
  // page table installed at low address space.
  fn start_first_process(&self) -> !;

  fn get_kernel_page_table(&self) -> PageTable;
}