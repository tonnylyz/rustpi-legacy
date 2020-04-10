use super::vm_descriptor::*;
use crate::mm::*;
use arch::*;

#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTable {
  directory: PageFrame
}

trait VirtualAddress {
  fn l1x(&self) -> usize;
  fn l2x(&self) -> usize;
  fn l3x(&self) -> usize;
}

impl VirtualAddress for usize {
  fn l1x(&self) -> usize {
    self >> PAGE_TABLE_L1_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l2x(&self) -> usize {
    self >> PAGE_TABLE_L2_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l3x(&self) -> usize {
    self >> PAGE_TABLE_L3_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
}

trait TableDescriptor {
  fn valid(&self) -> bool;
  fn get_entry(&self, index: usize) -> Self;
  fn set_entry(&self, index: usize, value: Self);
}

impl core::convert::From<ArchPageTableEntry> for PageTableEntry {
  fn from(u: ArchPageTableEntry) -> Self {
    use register::*;
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.to_u64());
    PageTableEntry{
      attr: PageTableEntryAttr {
        k_w: reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1) || reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0),
        k_x: !reg.is_set(PAGE_DESCRIPTOR::PXN),
        u_r: reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0) || reg.matches_all(PAGE_DESCRIPTOR::AP::RO_EL1_EL0),
        u_w: reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0),
        u_x: !reg.is_set(PAGE_DESCRIPTOR::UXN),
        copy_on_write: reg.is_set(PAGE_DESCRIPTOR::COW),
        shared: reg.is_set(PAGE_DESCRIPTOR::LIB),
        device: reg.matches_all(PAGE_DESCRIPTOR::AttrIndx::DEVICE)
      },
      addr: pte2pa(u.to_usize())
    }
  }
}

impl core::convert::From<PageTableEntry> for ArchPageTableEntry {
  fn from(pte: PageTableEntry) -> Self {
    ArchPageTableEntry::new((
      if pte.attr.shared { PAGE_DESCRIPTOR::LIB::True } else { PAGE_DESCRIPTOR::LIB::False }
        + if pte.attr.copy_on_write { PAGE_DESCRIPTOR::COW::True } else { PAGE_DESCRIPTOR::COW::False }
        + if pte.attr.u_x { PAGE_DESCRIPTOR::UXN::False } else { PAGE_DESCRIPTOR::UXN::True }
        + if pte.attr.k_x { PAGE_DESCRIPTOR::PXN::False } else { PAGE_DESCRIPTOR::PXN::True }
        + if pte.attr.device {
        PAGE_DESCRIPTOR::SH::OuterShareable + PAGE_DESCRIPTOR::AttrIndx::DEVICE
      } else {
        PAGE_DESCRIPTOR::SH::InnerShareable + PAGE_DESCRIPTOR::AttrIndx::NORMAL
      }
        + if pte.attr.k_w && pte.attr.u_w {
        PAGE_DESCRIPTOR::AP::RW_EL1_EL0
      } else if pte.attr.k_w && !pte.attr.u_r && !pte.attr.u_w {
        PAGE_DESCRIPTOR::AP::RW_EL1
      } else if !pte.attr.k_w && pte.attr.u_r && !pte.attr.u_w {
        PAGE_DESCRIPTOR::AP::RO_EL1_EL0
      } else if !pte.attr.k_w && !pte.attr.u_r && !pte.attr.u_w {
        PAGE_DESCRIPTOR::AP::RO_EL1
      } else {
        panic!("PAGE_DESCRIPTOR");
      }
        + PAGE_DESCRIPTOR::TYPE::Table
        + PAGE_DESCRIPTOR::VALID::True
        + PAGE_DESCRIPTOR::OUTPUT_ADDR.val((pte.addr >> 12) as u64)
        + PAGE_DESCRIPTOR::AF::True
    ).value)
  }
}

impl TableDescriptor for ArchPageTableEntry {
  fn valid(&self) -> bool {
    self.to_usize() & 0b11 != 0
  }
  fn get_entry(&self, index: usize) -> ArchPageTableEntry {
    let addr = pa2kva(pte2pa(self.to_usize()) + index * MACHINE_SIZE);
    unsafe { ArchPageTableEntry::new(core::intrinsics::volatile_load(addr as *const u64)) }
  }
  fn set_entry(&self, index: usize, value: ArchPageTableEntry) {
    let addr = pa2kva(pte2pa(self.to_usize()) + index * MACHINE_SIZE);
    unsafe { core::intrinsics::volatile_store(addr as *mut u64, value.to_u64()) }
  }
}

fn alloc_page_table() -> ArchPageTableEntry {
  let frame = crate::mm::page_pool::alloc();
  crate::mm::page_pool::increase_rc(frame);
  ArchPageTableEntry::from(PageTableEntry {
    attr: PageTableEntryAttr::readonly(),
    addr: frame.pa()
  })
}

impl PageTableImpl for Aarch64PageTable {
  fn new(directory: PageFrame) -> Self {
    Aarch64PageTable {
      directory
    }
  }

  fn directory(&self) -> PageFrame {
    self.directory
  }

  fn map(&self, va: usize, pa: usize, attr: PageTableEntryAttr) {
    let directory = ArchPageTableEntry::new(self.directory.pa() as u64);
    let mut l1e = directory.get_entry(va.l1x());
    if !l1e.valid() {
      l1e = alloc_page_table();
      directory.set_entry(va.l1x(), l1e);
    }
    let mut l2e = l1e.get_entry(va.l2x());
    if !l2e.valid() {
      l2e = alloc_page_table();
      l1e.set_entry(va.l2x(), l2e);
    }
    l2e.set_entry(va.l3x(), ArchPageTableEntry::from(PageTableEntry {
      attr,
      addr: pa,
    }));
  }

  fn unmap(&self, va: usize) {
    let directory = ArchPageTableEntry::new(self.directory.pa() as u64);
    let l1e = directory.get_entry(va.l1x());
    assert!(l1e.valid());
    let l2e = l1e.get_entry(va.l2x());
    assert!(l2e.valid());
    l2e.set_entry(va.l3x(), ArchPageTableEntry::new(0));
  }

  fn insert_page(&self, va: usize, frame: PageFrame, attr: PageTableEntryAttr) -> Result<(), PageTableError> {
    let pa = frame.pa();
    if let Some(p) = self.lookup_page(va) {
      if p.addr != pa {
        // replace mapped frame
        self.remove_page(va)?;
      } else {
        // update attribute
        ARCH.invalidate_tlb();
        self.map(va, pa, attr);
        return Ok(());
      }
    }
    ARCH.invalidate_tlb();
    self.map(va, pa, attr);
    crate::mm::page_pool::increase_rc(frame);
    Ok(())
  }

  fn lookup_page(&self, va: usize) -> Option<PageTableEntry> {
    let directory = ArchPageTableEntry::new(self.directory.pa() as u64);
    let l1e = directory.get_entry(va.l1x());
    if !l1e.valid() {
      return None;
    }
    let l2e = l1e.get_entry(va.l2x());
    if !l2e.valid() {
      return None;
    }
    let l3e = l2e.get_entry(va.l3x());
    if !(l3e.valid()) {
      None
    } else {
      Some(PageTableEntry::from(l3e))
    }
  }

  fn remove_page(&self, va: usize) -> Result<(), PageTableError> {
    if let Some(pte) = self.lookup_page(va) {
      let frame = PageFrame::new(pte.addr);
      crate::mm::page_pool::decrease_rc(frame);
      self.unmap(va);
      ARCH.invalidate_tlb();
      Ok(())
    } else {
      Err(PageTableError::VaNotMapped)
    }
  }

  fn recursive_map(&self, va: usize) {
    assert_eq!(va % (1 << PAGE_TABLE_L1_SHIFT), 0);
    let directory = ArchPageTableEntry::new(self.directory.pa() as u64);
    let l1x = va / (1 << PAGE_TABLE_L1_SHIFT);
    directory.set_entry(l1x, ArchPageTableEntry::from(PageTableEntry {
      attr: PageTableEntryAttr::readonly(),
      addr: self.directory.pa()
    }));
  }

  fn destroy(&self) {
    let directory = ArchPageTableEntry::new(self.directory.pa() as u64);
    for l1x in 0..(PAGE_SIZE / MACHINE_SIZE) {
      let l1e = directory.get_entry(l1x);
      if !l1e.valid() {
        continue;
      }
      for l2x in 0..(PAGE_SIZE / MACHINE_SIZE) {
        let l2e = l1e.get_entry(l2x);
        if !l2e.valid() {
          continue;
        }
        for l3x in 0..(PAGE_SIZE / MACHINE_SIZE) {
          let l3e = l2e.get_entry(l3x);
          if !l3e.valid() {
            continue;
          }
          let pa = pte2pa(l3e.to_usize());
          if crate::config::paged_range().contains(&pa) {
            let frame = PageFrame::new(pa);
            crate::mm::page_pool::decrease_rc(frame);
          }
        }
        let pa = pte2pa(l2e.to_usize());
        if crate::config::paged_range().contains(&pa) {
          let frame = PageFrame::new(pa);
          crate::mm::page_pool::decrease_rc(frame);
        }
      }
      let pa = pte2pa(l1e.to_usize());
      if crate::config::paged_range().contains(&pa) {
        let frame = PageFrame::new(pa);
        crate::mm::page_pool::decrease_rc(frame);
      }
    }
    // do not recycle directory page here!
  }
}
