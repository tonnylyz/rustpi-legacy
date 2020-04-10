#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(format_args_nl)]
#![feature(vec_remove_item)]
#![feature(asm)]

extern crate cortex_a;
extern crate register;
extern crate buddy_system_allocator;
extern crate alloc;
extern crate spin;
extern crate xmas_elf;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::lib::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::lib::print::print_arg(format_args_nl!($($arg)*));
    })
}

mod arch;
mod board;
mod driver;
mod lib;
mod mm;
mod config;

use arch::*;

#[no_mangle]
pub fn main() -> ! {
  driver::uart::init();
  ARCH.exception_init();
  mm::heap::init(config::heap_range());
  mm::page_pool::init(config::paged_range());
  driver::timer::init();
  lib::process_pool::init();

  unsafe {
    lib::process::Process::create(&lib::user_image::_binary_user_elf_start, 1);
    lib::scheduler::schedule();
  }
  // kernel mode -> user mode
  ARCH.start_first_process()
}
