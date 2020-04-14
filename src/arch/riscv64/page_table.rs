use crate::mm::PageFrame;
use crate::arch::ArchPageTableEntryTrait;
use crate::lib::page_table::{Entry, EntryAttribute, PageTableEntryAttrTrait, PageTableTrait, Error};

#[derive(Copy, Clone, Debug)]
pub struct Riscv64PageTable {
  directory: PageFrame
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64PageTableEntry(u64);

impl ArchPageTableEntryTrait for Riscv64PageTableEntry {
  fn new(value: usize) -> Self {
    unimplemented!()
  }

  fn to_usize(&self) -> usize {
    unimplemented!()
  }
}

impl core::convert::From<Riscv64PageTableEntry> for Entry {
  fn from(_: Riscv64PageTableEntry) -> Self {
    unimplemented!()
  }
}

impl core::convert::From<Entry> for Riscv64PageTableEntry {
  fn from(_: Entry) -> Self {
    unimplemented!()
  }
}

// TODO: remove this duplicated function
fn alloc_page_table() -> Riscv64PageTableEntry {
  let frame = crate::mm::page_pool::alloc();
  crate::mm::page_pool::increase_rc(frame);
  Riscv64PageTableEntry::from(Entry::new(EntryAttribute::user_readonly(), frame.pa()))
}

impl PageTableTrait for Riscv64PageTable {
  fn new(directory: PageFrame) -> Self {
    unimplemented!()
  }

  fn directory(&self) -> PageFrame {
    unimplemented!()
  }

  fn map(&self, va: usize, pa: usize, attr: EntryAttribute) {
    unimplemented!()
  }

  fn unmap(&self, va: usize) {
    unimplemented!()
  }

  fn insert_page(&self, va: usize, frame: PageFrame, attr: EntryAttribute) -> Result<(), Error> {
    unimplemented!()
  }

  fn lookup_page(&self, va: usize) -> Option<Entry> {
    unimplemented!()
  }

  fn remove_page(&self, va: usize) -> Result<(), Error> {
    unimplemented!()
  }

  fn recursive_map(&self, va: usize) {
    unimplemented!()
  }

  fn destroy(&self) {
    unimplemented!()
  }
}