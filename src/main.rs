#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(format_args_nl)]
#![feature(vec_remove_item)]
#![feature(const_in_array_repeat_expressions)]
#![feature(asm)]

extern crate alloc;
extern crate rlibc;

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

fn clear_bss() {
  extern "C" {
    fn BSS_START();
    fn BSS_END();
  }
  let start = pa2kva(BSS_START as usize);
  let end = pa2kva(BSS_END as usize);
  unsafe { core::intrinsics::volatile_set_memory(start as *mut u8, 0, end - start); }
}

fn static_check() {
  use config::*;
  use core::intrinsics::size_of;
  #[allow(unused_unsafe)]
  unsafe {
    assert_eq!(size_of::<crate::lib::process::Ipc>(), CONFIG_PROCESS_IPC_SIZE);
    // Note: size of ContextFrame needs to be synced with `arch/*/exception.S`
    if cfg!(target_arch = "aarch64") {
      assert_eq!(size_of::<ContextFrame>(), 0x110);
    } else if cfg!(target_arch = "riscv64") {
      assert_eq!(size_of::<ContextFrame>(), 0x110);
    } else {
      panic!("unsupported arch");
    }
  }
}

#[no_mangle]
pub fn main() -> ! {
  let core_id = arch::Arch::core_id();
  if core_id != 0 {
    loop { arch::Arch::wait_for_event(); } // capture other cores here
  }
  clear_bss();
  driver::uart::init();
  static_check();
  mm::heap::init(config::heap_range());
  mm::page_pool::init(config::paged_range());
  lib::process_pool::init();
  unsafe {
    // Note: `arg` is used to start different programs:
    //    0 - fktest: a `fork` test
    //    1 - pingpong: an IPC test
    //    2 - heap_test: test copy on write of heap
    #[cfg(target_arch = "aarch64")]
      lib::process::Process::create(&lib::user_image::_binary_user_aarch64_elf_start, 0);
    #[cfg(target_arch = "riscv64")]
      lib::process::Process::create(&lib::user_image::_binary_user_riscv64_elf_start, 0);
  }
  arch::Arch::exception_init();
  driver::timer::init(core_id);
  lib::scheduler::schedule();
  // kernel mode -> user mode
  arch::Arch::start_first_process()
}
