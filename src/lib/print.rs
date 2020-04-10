use core::fmt;

pub struct Writer;

static mut WRITER: Writer = Writer;

impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for b in s.bytes() {
      crate::driver::uart::putc(b);
    }
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
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  if let Some(m) = info.message() {
    println!("\nKernel panic: {} \n {}", m, info.location().unwrap());
  } else {
    println!("\nKernel panic!");
  }
  loop {
    cortex_a::asm::wfe();
  }
}