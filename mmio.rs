
pub fn mmio_read(ptr: usize) -> u32 {
	let p = ptr as *mut u32;
	unsafe {
		return *p;
	}
}

pub fn mmio_readb(ptr: usize) -> u8 {
	let p = ptr as *mut u8;
	unsafe {
		return *p;
	}
}

pub fn mmio_write(ptr: usize, val: u32) {
	let p = ptr as *mut u32;
	unsafe {
		*p = val;
	}
}


pub fn mmio_writeb(ptr: usize, val: u8) {
	let p = ptr as *mut u8;
	unsafe {
		*p = val;
	}
}
