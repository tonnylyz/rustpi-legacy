use core::fmt;

pub struct Writer {}

static mut WRITER: Writer = Writer {};

impl Writer {
  pub fn write_byte(&mut self, byte: u8) {
    crate::driver::uart::uart_putc(byte);
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


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
  loop {}
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

