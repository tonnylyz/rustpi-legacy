use cortex_a::{asm::*, regs::*};

use crate::arch::Address;
use crate::board::BOARD_CORE_NUMBER;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const MACHINE_SIZE: usize = 8;

const PA2KVA: usize = 0xFFFF_FF80_0000_0000;
const KVA2PA: usize = 0x0000_007F_FFFF_FFFF;

impl Address for usize {
  fn pa2kva(&self) -> usize {
    *self | PA2KVA
  }
  fn kva2pa(&self) -> usize {
    *self & KVA2PA
  }
}

pub type Arch = Aarch64Arch;

pub type ContextFrame = super::context_frame::Aarch64ContextFrame;

pub type PageTable = super::page_table::Aarch64PageTable;

pub type ArchPageTableEntry = super::page_table::Aarch64PageTableEntry;

pub type AddressSpaceId = u16;

pub struct Aarch64Arch;

impl crate::arch::ArchTrait for Aarch64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn invalidate_tlb() {
    unsafe {
      llvm_asm!("dsb ishst");
      llvm_asm!("tlbi vmalle1is");
      llvm_asm!("dsb ish");
      llvm_asm!("isb");
    }
  }

  fn wait_for_event() {
    wfe();
  }

  fn nop() {
    nop();
  }

  fn fault_address() -> usize {
    FAR_EL1.get() as usize
  }

  fn core_id() -> usize {
    MPIDR_EL1.get() as usize & (BOARD_CORE_NUMBER - 1)
  }
}