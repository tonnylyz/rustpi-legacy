pub const PAGE_SIZE: usize = 4096;

// TODO: modify these const for riscv64
pub const PA2KVA: usize = 0xFFFF_FF80_0000_0000;
pub const PTE2PA: usize = 0x0000_FFFF_FFFF_F000;
pub const KVA2PA: usize = 0xFFFF_FFFF;

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
  pte & PTE2PA
}

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}