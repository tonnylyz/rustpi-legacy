pub trait ContextFrameImpl: Default {
  fn get_syscall_argument(&self, i: usize) -> usize;
  fn get_syscall_number(&self) -> usize;
  fn set_syscall_return_value(&mut self, v: usize);
  fn get_exception_pc(&self) -> usize;
  fn set_exception_pc(&mut self, pc: usize);
  fn get_stack_pointer(&self) -> usize;
  fn set_stack_pointer(&mut self, sp: usize);
  fn set_argument(&mut self, arg: usize);
}

use crate::mm::PageFrame;
use arch::{PageTable, AddressSpaceId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PteAttribute {
  // Note: Execute indicates Read, Write indicates Read
  pub k_w: bool,
  pub k_x: bool,
  pub u_r: bool,
  pub u_w: bool,
  pub u_x: bool,
  pub copy_on_write: bool,
  pub shared: bool,
  pub device: bool,
}

impl core::ops::Add for PteAttribute {
  type Output = PteAttribute;

  fn add(self, rhs: Self) -> Self::Output {
    PteAttribute{
      k_w: self.k_w || rhs.k_w,
      k_x: self.k_x || rhs.k_x,
      u_r: self.u_r || rhs.u_r,
      u_w: self.u_w || rhs.u_w,
      u_x: self.u_x || rhs.u_x,
      copy_on_write: self.copy_on_write || rhs.copy_on_write,
      shared: self.shared || rhs.shared,
      device: self.device || rhs.device
    }
  }
}

impl core::ops::Sub for PteAttribute {
  type Output = PteAttribute;

  fn sub(self, rhs: Self) -> Self::Output {
    PteAttribute{
      k_w: self.k_w && !rhs.k_w,
      k_x: self.k_x && !rhs.k_x,
      u_r: self.u_r && !rhs.u_r,
      u_w: self.u_w && !rhs.u_w,
      u_x: self.u_x && !rhs.u_x,
      copy_on_write: self.copy_on_write && !rhs.copy_on_write,
      shared: self.shared && !rhs.shared,
      device: self.device && !rhs.device
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct PageTableEntry {
  pub attr: PteAttribute,
  pub addr: usize,
}

impl PteAttribute {
  pub const fn user_default() -> Self {
    PteAttribute {
      k_w: true,
      k_x: false,
      u_r: true,
      u_w: true,
      u_x: true,
      copy_on_write: false,
      shared: false,
      device: false,
    }
  }
  pub const fn kernel_device_default() -> Self {
    PteAttribute {
      k_w: true,
      k_x: false,
      u_r: false,
      u_w: false,
      u_x: false,
      copy_on_write: false,
      shared: false,
      device: true,
    }
  }
  pub const fn copy_on_write() -> Self {
    PteAttribute {
      k_w: false,
      k_x: false,
      u_r: false,
      u_w: false,
      u_x: false,
      copy_on_write: true,
      shared: false,
      device: false,
    }
  }
  pub const fn shared() -> Self {
    PteAttribute {
      k_w: false,
      k_x: false,
      u_r: false,
      u_w: false,
      u_x: false,
      copy_on_write: false,
      shared: true,
      device: false,
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum PageTableError {
  VaAlreadyMapped,
  VaNotMapped,
  VaRemoveFailed,
}

pub trait PageTableImpl {
  fn new(directory: PageFrame) -> Self;
  fn directory(&self) -> PageFrame;
  fn map(&self, va: usize, pa: usize, attr: PteAttribute);
  fn unmap(&self, va: usize);
  fn insert_page(&self, va: usize, frame: PageFrame, attr: PteAttribute) -> Result<(), PageTableError>;
  fn lookup_page(&self, va: usize) -> Option<PageTableEntry>;
  fn remove_page(&self, va: usize) -> Result<(), PageTableError>;
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
  fn get_user_page_table(&self) -> PageTable;
  fn set_user_page_table(&self, pt: PageTable, asid: AddressSpaceId);

  fn invalidate_tlb(&self);
}