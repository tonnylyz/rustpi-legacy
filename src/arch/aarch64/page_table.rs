use super::vm_descriptor::*;
use crate::mm::*;
use arch::*;

#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTable {
  directory: PageFrame
}


#[repr(C)]
struct Page([u64; PAGE_SIZE / 8]);

impl core::convert::From<PageFrame> for *mut Page {
  fn from(frame: PageFrame) -> Self {
    frame.kva() as *mut Page
  }
}


trait VirtualAddress {
  fn flx(&self) -> usize;
  // first level index
  fn slx(&self) -> usize;
  // second level index
  fn tlx(&self) -> usize; // third level index
}

impl VirtualAddress for usize {
  fn flx(&self) -> usize {
    self >> PAGE_TABLE_L1_SHIFT & (PAGE_SIZE - 1)
  }

  fn slx(&self) -> usize {
    self >> PAGE_TABLE_L2_SHIFT & (PAGE_SIZE - 1)
  }

  fn tlx(&self) -> usize {
    self >> PAGE_TABLE_L3_SHIFT & (PAGE_SIZE - 1)
  }
}

trait PageDescriptor {
  fn valid(&self) -> bool;
  fn next_level(&self) -> *mut Page;
}

impl PageDescriptor for u64 {
  fn valid(&self) -> bool {
    return self & 0b11 != 0;
  }
  fn next_level(&self) -> *mut Page {
    if !self.valid() {
      panic!("Invalid page descriptor");
    }
    let addr = *self as usize & 0xffff_ffff_f000;
    (addr | 0xFFFFFF8000000000) as *mut Page
  }
}

impl PageTableImpl for Aarch64PageTable {
  fn new(directory: PageFrame) -> Self {
    Aarch64PageTable {
      directory
    }
  }
  fn install(&self, pid: u16) {
    use cortex_a::{regs::*, *};
    // TODO: need mapping from pid to ASID
    let asid = pid;
    unsafe {
      TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64));
      TTBR0_EL1.set_baddr(self.directory.pa() as u64);
      barrier::isb(barrier::SY);
      barrier::dsb(barrier::SY);
    }
  }
  fn map(&self, va: usize, pa: usize) {
    unsafe {
      let directory_page: *mut Page = <*mut Page>::from(self.directory);
      let mut fle = (*directory_page).0[va.flx()]; // first level entry
      if !(fle.valid()) {
        let frame = crate::mm::page_pool::alloc();
        fle =
          (TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR.val((frame.pa() >> 12) as u64)
            + TABLE_DESCRIPTOR::TYPE::Table
            + TABLE_DESCRIPTOR::VALID::True
          ).value;
        (*directory_page).0[va.flx()] = fle;
      }
      let mut sle = (*fle.next_level()).0[va.slx()];
      if !(sle.valid()) {
        let frame = crate::mm::page_pool::alloc();
        sle =
          (TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR.val((frame.pa() >> 12) as u64)
            + TABLE_DESCRIPTOR::TYPE::Table
            + TABLE_DESCRIPTOR::VALID::True
          ).value;
        (*fle.next_level()).0[va.slx()] = sle;
      }
      let tle = (*sle.next_level()).0[va.tlx()];
      if tle.valid() {
        panic!("va already mapped")
      } else {
        (*sle.next_level()).0[va.tlx()] =
          (PAGE_DESCRIPTOR::PXN::True
            + PAGE_DESCRIPTOR::OUTPUT_ADDR.val((pa >> 12) as u64)
            + PAGE_DESCRIPTOR::AF::True
            + PAGE_DESCRIPTOR::SH::InnerShareable
            + PAGE_DESCRIPTOR::AP::RW_EL1_EL0
            + PAGE_DESCRIPTOR::AttrIndx::NORMAL
            + PAGE_DESCRIPTOR::TYPE::Table
            + PAGE_DESCRIPTOR::VALID::True
          ).value;
      }
    }
  }
  fn map_frame(&self, va: usize, frame: PageFrame) {
    let pa = frame.pa();
    self.map(va, pa);
  }

  fn unmap(&self, va: usize) {
    unimplemented!()
  }
}
