#[inline(always)]
#[allow(dead_code)]
pub unsafe fn read_word(ptr: usize) -> u32 {
  core::intrinsics::volatile_load(ptr as *mut u32)
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn read_byte(ptr: usize) -> u8 {
  core::intrinsics::volatile_load(ptr as *mut u8)
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn write_word(ptr: usize, val: u32) {
  core::intrinsics::volatile_store(ptr as *mut u32, val);
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn write_byte(ptr: usize, val: u8) {
  core::intrinsics::volatile_store(ptr as *mut u8, val);
}
