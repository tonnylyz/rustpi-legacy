
pub unsafe fn mmio_read(ptr: usize) -> u32 {
	let p = ptr as *mut u32;
	return *p;
}

pub unsafe fn mmio_readb(ptr: usize) -> u8 {
	let p = ptr as *mut u8;
	return *p;
}

pub unsafe fn mmio_write(ptr: usize, val: u32) {
	let p = ptr as *mut u32;
	*p = val;
}


pub unsafe fn mmio_writeb(ptr: usize, val: u8) {
	let p = ptr as *mut u8;
	*p = val;
}
