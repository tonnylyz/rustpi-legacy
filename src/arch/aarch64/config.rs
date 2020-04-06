pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

pub const PA2KVA: usize = 0xFFFF_FF80_0000_0000;
pub const PTE2PA: usize = 0x0000_FFFF_FFFF_F000;
pub const KVA2PA: usize = 0xFFFF_FFFF;
// helper function
#[inline(always)]
pub fn pa2kva(pa : usize) -> usize {
  pa | PA2KVA
}

#[inline(always)]
pub fn kva2pa(kva : usize) -> usize {
  kva & KVA2PA
}

#[inline(always)]
pub fn pte2pa(pte : usize) -> usize {
  pte & PTE2PA
}
