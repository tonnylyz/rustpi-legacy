use arch::*;
use lib::page_table::{Entry, EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use mm::PageFrame;

use super::vm_descriptor::*;

#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTable {
  directory: PageFrame
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTableEntry(u64);

impl ArchPageTableEntryTrait for Aarch64PageTableEntry {
  fn new(value: usize) -> Self {
    Aarch64PageTableEntry(value as u64)
  }

  fn to_usize(&self) -> usize {
    self.0 as usize
  }
}

trait Index {
  fn l1x(&self) -> usize;
  fn l2x(&self) -> usize;
  fn l3x(&self) -> usize;
}

impl Index for usize {
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

impl core::convert::From<Aarch64PageTableEntry> for Entry {
  fn from(u: Aarch64PageTableEntry) -> Self {
    use register::*;
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0);
    Entry::new(EntryAttribute::new(
      reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1) || reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0),
      reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0) || reg.matches_all(PAGE_DESCRIPTOR::AP::RO_EL1_EL0),
      reg.matches_all(PAGE_DESCRIPTOR::AttrIndx::DEVICE),
      !reg.is_set(PAGE_DESCRIPTOR::PXN),
      !reg.is_set(PAGE_DESCRIPTOR::UXN),
      reg.is_set(PAGE_DESCRIPTOR::COW),
      reg.is_set(PAGE_DESCRIPTOR::LIB),
    ), pte2pa(u.to_usize()))
  }
}

impl core::convert::From<Entry> for Aarch64PageTableEntry {
  fn from(pte: Entry) -> Self {
    Aarch64PageTableEntry((
      if pte.attribute().u_shared() { PAGE_DESCRIPTOR::LIB::True } else { PAGE_DESCRIPTOR::LIB::False }
        + if pte.attribute().u_copy_on_write() { PAGE_DESCRIPTOR::COW::True } else { PAGE_DESCRIPTOR::COW::False }
        + if pte.attribute().u_executable() { PAGE_DESCRIPTOR::UXN::False } else { PAGE_DESCRIPTOR::UXN::True }
        + if pte.attribute().k_executable() { PAGE_DESCRIPTOR::PXN::False } else { PAGE_DESCRIPTOR::PXN::True }
        + if pte.attribute().device() {
        PAGE_DESCRIPTOR::SH::OuterShareable + PAGE_DESCRIPTOR::AttrIndx::DEVICE
      } else {
        PAGE_DESCRIPTOR::SH::InnerShareable + PAGE_DESCRIPTOR::AttrIndx::NORMAL
      }
        + if pte.attribute().writable() && pte.attribute().u_readable() {
        PAGE_DESCRIPTOR::AP::RW_EL1_EL0
      } else if pte.attribute().writable() && !pte.attribute().u_readable() {
        PAGE_DESCRIPTOR::AP::RW_EL1
      } else if !pte.attribute().writable() && pte.attribute().u_readable() {
        PAGE_DESCRIPTOR::AP::RO_EL1_EL0
      } else {// if !pte.attr.writable() && !pte.attr.u_readable() {
        PAGE_DESCRIPTOR::AP::RO_EL1
      }
        + PAGE_DESCRIPTOR::TYPE::Table
        + PAGE_DESCRIPTOR::VALID::True
        + PAGE_DESCRIPTOR::OUTPUT_ADDR.val((pte.pa() >> PAGE_SHIFT) as u64)
        + PAGE_DESCRIPTOR::AF::True
    ).value)
  }
}

trait TableDescriptor {
  fn valid(&self) -> bool;
  fn get_entry(&self, index: usize) -> Self;
  fn set_entry(&self, index: usize, value: Self);
}

impl TableDescriptor for Aarch64PageTableEntry {
  fn valid(&self) -> bool {
    self.to_usize() & 0b11 != 0
  }
  fn get_entry(&self, index: usize) -> Aarch64PageTableEntry {
    let addr = pa2kva(pte2pa(self.to_usize()) + index * MACHINE_SIZE);
    unsafe { Aarch64PageTableEntry(core::intrinsics::volatile_load(addr as *const u64)) }
  }
  fn set_entry(&self, index: usize, value: Aarch64PageTableEntry) {
    let addr = pa2kva(pte2pa(self.to_usize()) + index * MACHINE_SIZE);
    unsafe { core::intrinsics::volatile_store(addr as *mut u64, value.0) }
  }
}

fn alloc_page_table() -> Aarch64PageTableEntry {
  let frame = crate::mm::page_pool::alloc();
  crate::mm::page_pool::increase_rc(frame);
  Aarch64PageTableEntry::from(Entry::new(EntryAttribute::user_readonly(), frame.pa()))
}

impl PageTableTrait for Aarch64PageTable {
  fn new(directory: PageFrame) -> Self {
    Aarch64PageTable {
      directory
    }
  }

  fn directory(&self) -> PageFrame {
    self.directory
  }

  fn map(&self, va: usize, pa: usize, attr: EntryAttribute) {
    let directory = Aarch64PageTableEntry(self.directory.pa() as u64);
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
    l2e.set_entry(va.l3x(), Aarch64PageTableEntry::from(Entry::new(attr, pa)));
  }

  fn unmap(&self, va: usize) {
    let directory = Aarch64PageTableEntry(self.directory.pa() as u64);
    let l1e = directory.get_entry(va.l1x());
    assert!(l1e.valid());
    let l2e = l1e.get_entry(va.l2x());
    assert!(l2e.valid());
    l2e.set_entry(va.l3x(), Aarch64PageTableEntry::new(0));
  }

  fn insert_page(&self, va: usize, frame: PageFrame, attr: EntryAttribute) -> Result<(), crate::lib::page_table::Error> {
    let pa = frame.pa();
    if let Some(p) = self.lookup_page(va) {
      if p.pa() != pa {
        // replace mapped frame
        self.remove_page(va)?;
      } else {
        // update attribute
        crate::arch::Arch::invalidate_tlb();
        self.map(va, pa, attr);
        return Ok(());
      }
    }
    crate::arch::Arch::invalidate_tlb();
    self.map(va, pa, attr);
    crate::mm::page_pool::increase_rc(frame);
    Ok(())
  }

  fn lookup_page(&self, va: usize) -> Option<Entry> {
    let directory = Aarch64PageTableEntry(self.directory.pa() as u64);
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
      Some(Entry::from(l3e))
    }
  }

  fn remove_page(&self, va: usize) -> Result<(), crate::lib::page_table::Error> {
    if let Some(pte) = self.lookup_page(va) {
      let frame = PageFrame::new(pte.pa());
      crate::mm::page_pool::decrease_rc(frame);
      self.unmap(va);
      crate::arch::Arch::invalidate_tlb();
      Ok(())
    } else {
      Err(crate::lib::page_table::Error::AddressNotMappedError)
    }
  }

  fn recursive_map(&self, va: usize) {
    assert_eq!(va % (1 << PAGE_TABLE_L1_SHIFT), 0);
    let directory = Aarch64PageTableEntry(self.directory.pa() as u64);
    let l1x = va / (1 << PAGE_TABLE_L1_SHIFT);
    directory.set_entry(l1x, Aarch64PageTableEntry::from(Entry::new(
      EntryAttribute::user_readonly(),
      self.directory.pa(),
    )));
  }

  fn destroy(&self) {
    let directory = Aarch64PageTableEntry(self.directory.pa() as u64);
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
  }
}
