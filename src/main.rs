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
    ($($arg:tt)*) => ($crate::lib::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::lib::print_arg(format_args_nl!($($arg)*));
    })
}

mod arch;
mod board;
mod driver;
mod lib;
mod mm;
mod config;

use arch::*;
use config::*;

use lib::user_image::*;
use lib::scheduler::{SCHEDULER, Scheduler};
use lib::process_pool::PROCESS_POOL;

#[no_mangle]
pub unsafe fn main() -> ! {
  driver::uart::uart_init();
  ARCH.exception_init();

  println!("Heap  Pool     {:08x}~{:08x}", heap_range().start, heap_range().end);
  println!("Paged Pool     {:08x}~{:08x}", paged_range().start, paged_range().end);
  mm::heap::init(heap_range());
  mm::page_pool::init(paged_range());

  driver::timer::timer_init();
  lib::process_pool::PROCESS_POOL.init();

  lib::process::Pid::create(&_binary_user_elf_start, 12345);
  lib::process::Pid::create(&_binary_user_elf_start, 54321);
  SCHEDULER.schedule(PROCESS_POOL.pid_list());
  // kernel mode -> user mode
  ARCH.start_first_process()
}
