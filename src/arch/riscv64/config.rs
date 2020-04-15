pub const PAGE_SIZE: usize = 4096;

const PA2KVA: usize = 0xFFFF_FFFF_0000_0000;
const KVA2PA: usize = 0xFFFF_FFFF;

// helper function
#[inline(always)]
pub fn pa2kva(pa: usize) -> usize {
  pa | PA2KVA
}

#[inline(always)]
pub fn kva2pa(kva: usize) -> usize {
  kva & KVA2PA
}

#[inline(always)]
pub fn pte2pa(pte: usize) -> usize {
  ((pte >> 10) << 12) & KVA2PA
}

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}