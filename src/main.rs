#![no_std]
#![no_main]

extern crate cortex_a;

mod driver;
mod lib;
use lib::print;

#[no_mangle]
pub fn main(freemem : usize) -> ! {
  driver::uart::uart_init();
  println!("Freemem @ 0x{:x}", freemem);
  loop {}
}
