use core::fmt;
use crate::uart::uart_putc;
pub struct Writer {

}

static mut WRITER: Writer = Writer {};

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                uart_putc(b'\r');
                uart_putc(b'\n');
			}
            byte => { uart_putc(byte) }
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

pub fn print_arg(args: fmt::Arguments) {
    use core::fmt::Write;
	unsafe {
    	WRITER.write_fmt(args).unwrap();
	}
}


