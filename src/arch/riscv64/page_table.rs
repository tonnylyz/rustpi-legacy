use crate::mm::PageFrame;
use crate::arch::{ArchPageTableEntryTrait, pte2pa, PAGE_SHIFT, pa2kva, MACHINE_SIZE, PAGE_SIZE, PAGE_TABLE_L1_SHIFT, PAGE_TABLE_L2_SHIFT, PAGE_TABLE_L3_SHIFT, ArchTrait, pa2pte};
use crate::lib::page_table::{Entry, EntryAttribute, PageTableEntryAttrTrait, PageTableTrait, Error};

use super::vm_descriptor::*;

#[derive(Copy, Clone, Debug)]
pub struct Riscv64PageTable {
  directory: PageFrame
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64PageTableEntry(u64);

impl ArchPageTableEntryTrait for Riscv64PageTableEntry {
  fn new(value: usize) -> Self {
    Riscv64PageTableEntry(value as u64)
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

impl core::convert::From<Riscv64PageTableEntry> for Entry {
  fn from(u: Riscv64PageTableEntry) -> Self {
    use register::*;
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0);
    Entry::new(EntryAttribute::new(
      reg.is_set(PAGE_DESCRIPTOR::W),
      reg.is_set(PAGE_DESCRIPTOR::USER),
      false, // riscv do not has bits indicating device memory
      false, // reg.is_set(PAGE_DESCRIPTOR::X)  && SUM bit in sstatus
      reg.is_set(PAGE_DESCRIPTOR::X),
      reg.is_set(PAGE_DESCRIPTOR::COW),
      reg.is_set(PAGE_DESCRIPTOR::LIB),
    ), pte2pa(u.to_usize()))
  }
}

impl core::convert::From<Entry> for Riscv64PageTableEntry {
  fn from(pte: Entry) -> Self {
    let r = Riscv64PageTableEntry((
      if pte.attribute().u_shared() { PAGE_DESCRIPTOR::LIB::True } else { PAGE_DESCRIPTOR::LIB::False }
        + if pte.attribute().u_copy_on_write() { PAGE_DESCRIPTOR::COW::True } else { PAGE_DESCRIPTOR::COW::False }
        + if pte.attribute().u_executable() { PAGE_DESCRIPTOR::X::True } else { PAGE_DESCRIPTOR::X::False }
        + if pte.attribute().u_readable() { PAGE_DESCRIPTOR::R::True } else { PAGE_DESCRIPTOR::R::False }
        + if pte.attribute().writable() { PAGE_DESCRIPTOR::W::True } else { PAGE_DESCRIPTOR::W::False }
        + PAGE_DESCRIPTOR::DIRTY::True
        + PAGE_DESCRIPTOR::ACCESSED::True
        + PAGE_DESCRIPTOR::VALID::True
        + PAGE_DESCRIPTOR::USER::True
        + PAGE_DESCRIPTOR::OUTPUT_PPN.val((pte.pa() >> PAGE_SHIFT) as u64)
    ).value);
    println!("leaf pte: {:016x}", r.0);
    r
  }
}

trait TableDescriptor {
  fn valid(&self) -> bool;
  fn get_entry(&self, index: usize) -> Self;
  fn set_entry(&self, index: usize, value: Self);
}

impl TableDescriptor for Riscv64PageTableEntry {
  fn valid(&self) -> bool {
    self.to_usize() & 0b1 != 0
  }
  fn get_entry(&self, index: usize) -> Riscv64PageTableEntry {
    let addr = pa2kva(pte2pa(self.to_usize()) + index * MACHINE_SIZE);
    unsafe { Riscv64PageTableEntry(core::intrinsics::volatile_load(addr as *const u64)) }
  }
  fn set_entry(&self, index: usize, value: Riscv64PageTableEntry) {
    let addr = pa2kva(pte2pa(self.to_usize()) + index * MACHINE_SIZE);
    unsafe { core::intrinsics::volatile_store(addr as *mut u64, value.0) }
  }
}

fn alloc_page_table() -> Riscv64PageTableEntry {
  let frame = crate::mm::page_pool::alloc();
  crate::mm::page_pool::increase_rc(frame);
  //Riscv64PageTableEntry::from(Entry::new(EntryAttribute::user_readonly(), frame.pa()))
  let r = Riscv64PageTableEntry::new(((frame.pa() >> PAGE_SHIFT) << 10) | 0b11010001);
  println!("alloc_page_table: {:016x}", r.0);
  r
}

impl PageTableTrait for Riscv64PageTable {
  fn new(directory: PageFrame) -> Self {
    // TODO: Copy high address kernel page directory
    Riscv64PageTable {
      directory
    }
  }

  fn directory(&self) -> PageFrame {
    self.directory
  }

  fn map(&self, va: usize, pa: usize, attr: EntryAttribute) {
    let directory = Riscv64PageTableEntry(pa2pte(self.directory.pa()) as u64);
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
    l2e.set_entry(va.l3x(), Riscv64PageTableEntry::from(Entry::new(attr, pa)));
  }

  fn unmap(&self, va: usize) {
    let directory = Riscv64PageTableEntry(pa2pte(self.directory.pa()) as u64);
    let l1e = directory.get_entry(va.l1x());
    assert!(l1e.valid());
    let l2e = l1e.get_entry(va.l2x());
    assert!(l2e.valid());
    l2e.set_entry(va.l3x(), Riscv64PageTableEntry::new(0));
  }

  fn insert_page(&self, va: usize, frame: PageFrame, attr: EntryAttribute) -> Result<(), Error> {
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
    let directory = Riscv64PageTableEntry(pa2pte(self.directory.pa()) as u64);
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
    // TODO: workaround needed
  }

  fn destroy(&self) {
    let directory = Riscv64PageTableEntry(pa2pte(self.directory.pa()) as u64);
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
