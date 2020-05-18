#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(format_args_nl)]
#![feature(vec_remove_item)]
#![feature(const_in_array_repeat_expressions)]
#![feature(llvm_asm)]

extern crate alloc;
#[macro_use]
extern crate lazy_static;
extern crate rlibc;

use arch::*;

use crate::lib::current_thread;

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
  let start = (BSS_START as usize).pa2kva();
  let end = (BSS_END as usize).pa2kva();
  unsafe { core::intrinsics::volatile_set_memory(start as *mut u8, 0, end - start); }
}

fn static_check() {
  use core::intrinsics::size_of;
  #[allow(unused_unsafe)]
    unsafe {
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
pub unsafe fn main(core_id: CoreId) -> ! {
  if core_id == 0 {
    clear_bss();
    board::init();
    static_check();
    mm::heap::init();
    mm::page_pool::init();
    board::launch_other_cores();
  }
  board::init_per_core();
  if core_id == 0 {
    // Note: `arg` is used to start different programs:
    //    0 - fktest: a `fork` test
    //    1 - pingpong: an IPC test
    //    2 - heap_test: test copy on write of heap
    #[cfg(target_arch = "aarch64")]
      lib::process::create(&lib::user_image::_binary_user_aarch64_elf_start, 0);
    #[cfg(target_arch = "riscv64")]
      lib::process::create(&lib::user_image::_binary_user_riscv64_elf_start, 0);
  }

  lib::scheduler::schedule();
  start_first_thread(core_id)
}

fn start_first_thread(core_id: CoreId) -> ! {
  extern {
    fn pop_context_first(ctx: usize, core_id: usize) -> !;
  }
  let t = current_thread().unwrap();
  let lock = t.context();
  let ctx_on_stack = *lock;
  drop(lock);
  unsafe {
    pop_context_first(&ctx_on_stack as *const _ as usize, core_id);
  }
}
