#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(format_args_nl)]
#![feature(vec_remove_item)]
#![feature(asm)]

extern crate alloc;
extern crate buddy_system_allocator;
extern crate cortex_a;
extern crate register;
extern crate spin;
#[macro_use]
extern crate static_assertions;
extern crate xmas_elf;


use arch::*;

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

fn static_check() {
  use config::*;
  use core::intrinsics::size_of;
  const_assert_eq!(size_of::<crate::lib::process::Ipc>(), CONFIG_PROCESS_IPC_SIZE);
  const_assert!(size_of::<ContextFrame>() < PAGE_SIZE);
}

#[no_mangle]
pub fn main() -> ! {
  driver::uart::init();
  static_check();

  arch::Arch::exception_init();
  mm::heap::init(config::heap_range());
  mm::page_pool::init(config::paged_range());
  driver::timer::init();
  lib::process_pool::init();

  unsafe {
    // Note: `arg` is used to start different programs:
    //    0 - fktest: a `fork` test
    //    1 - pingpong: an IPC test
    //    2 - heap_test: test copy on write of heap
    lib::process::Process::create(&lib::user_image::_binary_user_elf_start, 0);
  }
  lib::scheduler::schedule();
  // kernel mode -> user mode
  arch::Arch::start_first_process()
}
