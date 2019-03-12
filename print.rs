use core::fmt;

pub struct Writer {

}

static mut WRITER: Writer = Writer {};

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
				putc(b'\r');
				putc(b'\n');
			}
            byte => putc(byte)
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

extern {
	fn nop();
}

fn safe_nop() {
	unsafe {
		nop();
	}
}

pub fn putc(c: u8) {
	while (super::mmio::mmio_read(0x3f201018) & 0x20) != 0 {
		safe_nop();
	}
	super::mmio::mmio_writeb(0x3f201000, c);
}

pub fn print_arg(args: fmt::Arguments) {
    use core::fmt::Write;
	unsafe {
    	WRITER.write_fmt(args).unwrap();
	}
}


