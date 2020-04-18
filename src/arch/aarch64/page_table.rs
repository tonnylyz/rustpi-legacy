use crate::arch::*;
use crate::lib::page_table::{Entry, EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::mm::PageFrame;

use super::vm_descriptor::*;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTable {
  directory: PageFrame
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTableEntry(usize);

impl ArchPageTableEntryTrait for Aarch64PageTableEntry {
  fn from_pte(value: usize) -> Self {
    Aarch64PageTableEntry(value)
  }

  fn from_pa(pa: usize) -> Self {
    Aarch64PageTableEntry(pa)
  }

  fn to_pte(&self) -> usize {
    self.0
  }

  fn to_pa(&self) -> usize {
    self.0 & 0x0000_FFFF_FFFF_F000
  }

  fn to_kva(&self) -> usize {
    self.to_pa().pa2kva()
  }

  fn valid(&self) -> bool {
    self.0 & 0b11 != 0
  }

  fn entry(&self, index: usize) -> Aarch64PageTableEntry {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { Aarch64PageTableEntry(core::intrinsics::volatile_load(addr as *const usize)) }
  }

  fn set_entry(&self, index: usize, value: Aarch64PageTableEntry) {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { core::intrinsics::volatile_store(addr as *mut usize, value.0) }
  }

  fn alloc_table() -> Self {
    let frame = crate::mm::page_pool::alloc();
    crate::mm::page_pool::increase_rc(frame);
    Aarch64PageTableEntry::from(Entry::new(EntryAttribute::user_readonly(), frame.pa()))
  }
}

trait Index {
  fn l1x(&self) -> usize;
  fn l2x(&self) -> usize;
  fn l3x(&self) -> usize;
}

impl Index for usize {
  fn l1x(&self) -> usize {
    (self >> PAGE_TABLE_L1_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l2x(&self) -> usize {
    (self >> PAGE_TABLE_L2_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l3x(&self) -> usize {
    (self >> PAGE_TABLE_L3_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
}

impl core::convert::From<Aarch64PageTableEntry> for Entry {
  fn from(u: Aarch64PageTableEntry) -> Self {
    use register::*;
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0 as u64);
    Entry::new(EntryAttribute::new(
      reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1) || reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0),
      reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0) || reg.matches_all(PAGE_DESCRIPTOR::AP::RO_EL1_EL0),
      reg.matches_all(PAGE_DESCRIPTOR::AttrIndx::DEVICE),
      !reg.is_set(PAGE_DESCRIPTOR::PXN),
      !reg.is_set(PAGE_DESCRIPTOR::UXN),
      reg.is_set(PAGE_DESCRIPTOR::COW),
      reg.is_set(PAGE_DESCRIPTOR::LIB),
    ), (reg.read(PAGE_DESCRIPTOR::OUTPUT_PPN) as usize) << PAGE_SHIFT)
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
        + PAGE_DESCRIPTOR::OUTPUT_PPN.val((pte.ppn()) as u64)
        + PAGE_DESCRIPTOR::AF::True
    ).value as usize)
  }
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
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let mut l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      l1e = Aarch64PageTableEntry::alloc_table();
      directory.set_entry(va.l1x(), l1e);
    }
    let mut l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      l2e = Aarch64PageTableEntry::alloc_table();
      l1e.set_entry(va.l2x(), l2e);
    }
    l2e.set_entry(va.l3x(), Aarch64PageTableEntry::from(Entry::new(attr, pa)));
  }

  fn unmap(&self, va: usize) {
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let l1e = directory.entry(va.l1x());
    assert!(l1e.valid());
    let l2e = l1e.entry(va.l2x());
    assert!(l2e.valid());
    l2e.set_entry(va.l3x(), Aarch64PageTableEntry(0));
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
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      return None;
    }
    let l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      return None;
    }
    let l3e = l2e.entry(va.l3x());
    if l3e.valid() {
      Some(Entry::from(l3e))
    } else {
      None
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
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let l1x = va / (1 << PAGE_TABLE_L1_SHIFT);
    directory.set_entry(l1x, Aarch64PageTableEntry::from(Entry::new(
      EntryAttribute::user_readonly(),
      self.directory.pa(),
    )));
  }

  fn destroy(&self) {
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    for l1x in 0..(PAGE_SIZE / MACHINE_SIZE) {
      let l1e = directory.entry(l1x);
      if !l1e.valid() {
        continue;
      }
      for l2x in 0..(PAGE_SIZE / MACHINE_SIZE) {
        let l2e = l1e.entry(l2x);
        if !l2e.valid() {
          continue;
        }
        for l3x in 0..(PAGE_SIZE / MACHINE_SIZE) {
          let l3e = l2e.entry(l3x);
          if !l3e.valid() {
            continue;
          }
          let pa = l3e.to_pa();
          if crate::mm::config::paged_range().contains(&pa) {
            let frame = PageFrame::new(pa);
            crate::mm::page_pool::decrease_rc(frame);
          }
        }
        let pa = l2e.to_pa();
        if crate::mm::config::paged_range().contains(&pa) {
          let frame = PageFrame::new(pa);
          crate::mm::page_pool::decrease_rc(frame);
        }
      }
      let pa = l1e.to_pa();
      if crate::mm::config::paged_range().contains(&pa) {
        let frame = PageFrame::new(pa);
        crate::mm::page_pool::decrease_rc(frame);
      }
    }
  }

  fn kernel_page_table() -> PageTable {
    let frame = PageFrame::new(cortex_a::regs::TTBR1_EL1.get_baddr() as usize);
    PageTable::new(frame)
  }

  fn user_page_table() -> PageTable {
    let frame = PageFrame::new(cortex_a::regs::TTBR0_EL1.get_baddr() as usize);
    PageTable::new(frame)
  }

  fn set_user_page_table(pt: PageTable, asid: AddressSpaceId) {
    use cortex_a::{regs::*, *};
    unsafe {
      TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64));
      TTBR0_EL1.set_baddr(pt.directory().pa() as u64);
      barrier::isb(barrier::SY);
      barrier::dsb(barrier::SY);
    }
  }
}
