const BITMAP_SIZE: usize = 65536;
const BITMAP_ATOMIC_SIZE: usize = 64;

type BitmapAtomicType = u64;

pub struct BitMap([BitmapAtomicType; BITMAP_SIZE / BITMAP_ATOMIC_SIZE]);

impl BitMap {
  pub const fn new() -> Self {
    BitMap([0; BITMAP_SIZE / BITMAP_ATOMIC_SIZE])
  }

  pub fn set(&mut self, index: usize) {
    let i = index / BITMAP_ATOMIC_SIZE;
    let shift = index % BITMAP_ATOMIC_SIZE;
    self.0[i] |= (1usize << shift) as BitmapAtomicType;
  }

  pub fn clear(&mut self, index: usize) {
    let i = index / BITMAP_ATOMIC_SIZE;
    let shift = index % BITMAP_ATOMIC_SIZE;
    self.0[i] &= !(1usize << shift) as BitmapAtomicType;
  }

  #[allow(dead_code)]
  pub fn is_set(&self, index: usize) -> bool {
    let i = index / BITMAP_ATOMIC_SIZE;
    let shift = index % BITMAP_ATOMIC_SIZE;
    (self.0[i] >> shift as BitmapAtomicType) & 0b1 == 0b1
  }

  pub fn alloc(&mut self) -> usize {
    for i in 0..(BITMAP_SIZE / BITMAP_ATOMIC_SIZE) {
      let atom = self.0[i];
      if atom == BitmapAtomicType::MAX {
        continue;
      }
      for shift in 0..BITMAP_ATOMIC_SIZE {
        if (atom >> shift as BitmapAtomicType) & 0b1 == 0 {
          let index = i * BITMAP_ATOMIC_SIZE + shift;
          self.set(index);
          return index;
        }
      }
    }
    panic!("bitmap: out of zero bit");
  }
}