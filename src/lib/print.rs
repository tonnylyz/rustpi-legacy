use core::fmt;

use crate::arch::*;

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
        if let Some(l) = info.location() {
            println!("\nkernel panic: {} \n {}", m, l);
        } else {
            println!("\nkernel panic: {}", m);
        }
    } else {
        println!("\nkernel panic!");
    }
    loop {
        Arch::wait_for_event();
    }
}
