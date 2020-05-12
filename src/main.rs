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

fn kthread_test(arg: usize) {
  loop {
    print!("{}", arg);
  }
}

#[no_mangle]
pub unsafe fn main() -> ! {
  clear_bss();
  board::init();
  static_check();
  mm::heap::init();
  mm::page_pool::init();
  board::init_per_core();
  // Note: `arg` is used to start different programs:
  //    0 - fktest: a `fork` test
  //    1 - pingpong: an IPC test
  //    2 - heap_test: test copy on write of heap
  #[cfg(target_arch = "aarch64")]
    lib::process::create(&lib::user_image::_binary_user_aarch64_elf_start, 0);
  #[cfg(target_arch = "riscv64")]
    lib::process::create(&lib::user_image::_binary_user_riscv64_elf_start, 0);
  let t = lib::thread::alloc_kernel(kthread_test as usize, mm::page_pool::alloc().kva() + PAGE_SIZE, 0);
  t.set_status(lib::thread::Status::TsRunnable);
  // let u = lib::thread::alloc_kernel(kthread_test as usize, mm::page_pool::alloc().kva() + PAGE_SIZE, 1);
  // u.set_status(lib::thread::Status::TsRunnable);
  arch::Arch::exception_init();
  driver::timer::init(0);
  lib::scheduler::schedule();
  extern {
    fn pop_context_first(ctx: usize) -> !;
  }
  let t = lib::current_thread().unwrap();
  let ctx = *t.context();
  pop_context_first(&ctx as *const _ as usize);
}
