use riscv::regs::*;

use crate::arch::*;
use crate::config::*;
use crate::lib::page_table::{Entry, EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::mm::PageFrame;

use super::vm_descriptor::*;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

#[derive(Copy, Clone, Debug)]
pub struct Riscv64PageTable {
  directory: PageFrame
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64PageTableEntry(usize);

impl ArchPageTableEntryTrait for Riscv64PageTableEntry {
  fn from_pte(value: usize) -> Self {
    Riscv64PageTableEntry(value)
  }

  fn from_pa(pa: usize) -> Self {
    Riscv64PageTableEntry((pa >> 12) << 10)
  }

  fn to_pte(&self) -> usize {
    self.0
  }

  fn to_pa(&self) -> usize {
    (self.0 >> 10) << 12
  }

  fn to_kva(&self) -> usize {
    self.to_pa().pa2kva()
  }

  fn valid(&self) -> bool {
    // V and NOT RWX
    self.0 & 0b1 != 0
  }

  fn entry(&self, index: usize) -> Self {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { Riscv64PageTableEntry(core::intrinsics::volatile_load(addr as *const usize)) }
  }

  fn set_entry(&self, index: usize, value: Self) {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { core::intrinsics::volatile_store(addr as *mut usize, value.0) }
  }

  fn alloc_table() -> Self {
    let frame = crate::mm::page_pool::alloc();
    crate::mm::page_pool::increase_rc(frame);
    Riscv64PageTableEntry(
      (TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_PPN.val((frame.pa() >> PAGE_SHIFT) as u64)
        + TABLE_DESCRIPTOR::DIRTY::True
        + TABLE_DESCRIPTOR::ACCESSED::True
        + TABLE_DESCRIPTOR::USER::True
        + TABLE_DESCRIPTOR::VALID::True
      ).value as usize
    )
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
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0 as u64);
    Entry::new(EntryAttribute::new(
      reg.is_set(PAGE_DESCRIPTOR::W),
      reg.is_set(PAGE_DESCRIPTOR::USER),
      false, // riscv do not has bits indicating device memory
      false, // reg.is_set(PAGE_DESCRIPTOR::X)  && SUM bit in sstatus
      reg.is_set(PAGE_DESCRIPTOR::X),
      reg.is_set(PAGE_DESCRIPTOR::COW),
      reg.is_set(PAGE_DESCRIPTOR::LIB),
    ), (reg.read(PAGE_DESCRIPTOR::OUTPUT_PPN) as usize) << PAGE_SHIFT)
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
        + PAGE_DESCRIPTOR::OUTPUT_PPN.val((pte.ppn()) as u64)
    ).value as usize);
    r
  }
}

impl Riscv64PageTable {
  fn map_kernel_gigabyte_page(&self, va: usize, pa: usize) {
    let l1x = va.l1x();
    let directory = Riscv64PageTableEntry::from_pa(self.directory.pa());
    // same as mapping in `start.S`
    directory.set_entry(l1x, Riscv64PageTableEntry(
      (PAGE_DESCRIPTOR::OUTPUT_PPN.val((pa >> PAGE_SHIFT) as u64)
        + PAGE_DESCRIPTOR::DIRTY::True
        + PAGE_DESCRIPTOR::ACCESSED::True
        + PAGE_DESCRIPTOR::USER::False
        + PAGE_DESCRIPTOR::X::True
        + PAGE_DESCRIPTOR::W::True
        + PAGE_DESCRIPTOR::R::True
        + PAGE_DESCRIPTOR::VALID::True).value as usize
    ));
  }
}

impl PageTableTrait for Riscv64PageTable {
  fn new(directory: PageFrame) -> Self {
    let r = Riscv64PageTable {
      directory
    };
    r.map_kernel_gigabyte_page(0xffff_ffff_0000_0000, 0x0000_0000);
    r.map_kernel_gigabyte_page(0xffff_ffff_4000_0000, 0x4000_0000);
    r.map_kernel_gigabyte_page(0xffff_ffff_8000_0000, 0x8000_0000);
    r.map_kernel_gigabyte_page(0xffff_ffff_c000_0000, 0xc000_0000);
    r
  }

  fn directory(&self) -> PageFrame {
    self.directory
  }

  fn map(&self, va: usize, pa: usize, attr: EntryAttribute) {
    let directory = Riscv64PageTableEntry::from_pa(self.directory.pa());
    let mut l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      l1e = Riscv64PageTableEntry::alloc_table();
      if va <= CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM {
        self.map(CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM + va.l1x() * PAGE_SIZE, l1e.to_pa(), EntryAttribute::user_readonly());
      }
      directory.set_entry(va.l1x(), l1e);
    }
    let mut l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      l2e = Riscv64PageTableEntry::alloc_table();
      if va <= CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM {
        self.map(CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM + va.l1x() * PAGE_SIZE * (PAGE_SIZE / MACHINE_SIZE) + va.l2x() * PAGE_SIZE, l2e.to_pa(), EntryAttribute::user_readonly());
      }
      l1e.set_entry(va.l2x(), l2e);
    }
    l2e.set_entry(va.l3x(), Riscv64PageTableEntry::from(Entry::new(attr, pa)));
  }

  fn unmap(&self, va: usize) {
    let directory = Riscv64PageTableEntry::from_pa(self.directory.pa());
    let l1e = directory.entry(va.l1x());
    assert!(l1e.valid());
    let l2e = l1e.entry(va.l2x());
    assert!(l2e.valid());
    l2e.set_entry(va.l3x(), Riscv64PageTableEntry(0));
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
    let directory = Riscv64PageTableEntry::from_pa(self.directory.pa());
    let l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      return None;
    }
    let l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      return None;
    }
    let l3e = l2e.entry(va.l3x());
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

  fn recursive_map(&self, _va: usize) {
    self.map(CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM, self.directory.pa(), EntryAttribute::user_readonly());
  }

  fn destroy(&self) {
    let directory = Riscv64PageTableEntry::from_pa(self.directory.pa());
    for l1x in 0..(PAGE_SIZE / MACHINE_SIZE) {
      let l1e = directory.entry(l1x);
      if !l1e.valid() || l1e.to_pte() & 0b1110 != 0 {
        // Note: level 1 has gigabyte pages
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
    let ppn = SATP.read(SATP::PPN) as usize;
    PageTable::new(PageFrame::new(ppn << PAGE_SHIFT))
  }

  fn user_page_table() -> PageTable {
    // Note user and kernel share page directory
    Self::kernel_page_table()
  }

  fn set_user_page_table(pt: PageTable, asid: AddressSpaceId) {
    SATP.write(SATP::MODE::Sv39 + SATP::ASID.val(asid as u64) + SATP::PPN.val((pt.directory().pa() >> PAGE_SHIFT) as u64));
    riscv::barrier::sfence_vma_all();
  }
}
