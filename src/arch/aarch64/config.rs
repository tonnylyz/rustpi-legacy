pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

pub const KERNEL_HIGH_ADDRESS_START: usize = 0xFFFFFF8000000000;

// helper function
#[inline(always)]
pub fn pa2kva(pa : usize) -> usize {
  KERNEL_HIGH_ADDRESS_START | pa
}

#[inline(always)]
pub fn kva2pa(kva : usize) -> usize {
  kva & 0xFFFF_FFFF
}
