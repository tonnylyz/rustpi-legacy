#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod mmio;
mod print;
mod uart;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
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


#[no_mangle]
pub fn main() -> ! {
    uart::uart_init();

	println!("Hello {}", "world!");
    loop {}
}
