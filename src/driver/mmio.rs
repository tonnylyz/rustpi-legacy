
#[inline(always)]
pub unsafe fn mmio_read(ptr: usize) -> u32 {
	let p = ptr as *mut u32;
	return *p;
}

#[inline(always)]
pub unsafe fn mmio_readb(ptr: usize) -> u8 {
	let p = ptr as *mut u8;
	return *p;
}

#[inline(always)]
pub unsafe fn mmio_write(ptr: usize, val: u32) {
	let p = ptr as *mut u32;
	*p = val;
}

#[inline(always)]
pub unsafe fn mmio_writeb(ptr: usize, val: u8) {
	let p = ptr as *mut u8;
	*p = val;
}
