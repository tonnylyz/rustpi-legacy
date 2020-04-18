use riscv::{asm::*, regs::*};

use crate::arch::Address;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const MACHINE_SIZE: usize = 8;

const PA2KVA: usize = 0xFFFF_FFFF_0000_0000;
const KVA2PA: usize = 0xFFFF_FFFF;

impl Address for usize {
  fn pa2kva(&self) -> usize {
    *self | PA2KVA
  }
  fn kva2pa(&self) -> usize {
    *self & KVA2PA
  }
}

pub type Arch = Riscv64Arch;

pub type ContextFrame = super::context_frame::Riscv64ContextFrame;

pub type PageTable = super::page_table::Riscv64PageTable;

pub type ArchPageTableEntry = super::page_table::Riscv64PageTableEntry;

pub type AddressSpaceId = u16;

pub type Core = super::super::common::core::Core;

pub struct Riscv64Arch;

impl crate::arch::ArchTrait for Riscv64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn invalidate_tlb() {
    riscv::barrier::sfence_vma_all();
  }

  fn wait_for_event() {
    wfi();
  }

  fn nop() {
    nop();
  }

  fn fault_address() -> usize {
    STVAL.get() as usize
  }

  fn core_id() -> usize {
    // TODO: (riscv64) core id
    0
  }
}