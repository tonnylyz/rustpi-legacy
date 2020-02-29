#[inline(always)]
#[allow(dead_code)]
pub unsafe fn mmio_read(ptr: usize) -> u32 {
  core::intrinsics::volatile_load(ptr as *mut u32)
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn mmio_readb(ptr: usize) -> u8 {
  core::intrinsics::volatile_load(ptr as *mut u8)
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn mmio_write(ptr: usize, val: u32) {
  core::intrinsics::volatile_store(ptr as *mut u32, val);
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn mmio_writeb(ptr: usize, val: u8) {
  core::intrinsics::volatile_store(ptr as *mut u8, val);
}
