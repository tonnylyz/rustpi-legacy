#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(format_args_nl)]
#![feature(vec_remove_item)]

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
use lib::elf::read_elf;

#[no_mangle]
pub unsafe fn main() -> ! {
  driver::uart::uart_init();
  ARCH.exception_init();

  println!("Heap  Pool     {:08x}~{:08x}", heap_range().start, heap_range().end);
  println!("Paged Pool     {:08x}~{:08x}", paged_range().start, paged_range().end);
  mm::heap::init(heap_range());
  mm::page_pool::init(paged_range());

  driver::timer::timer_init();


  let p1 = lib::process::process_alloc();
  let page_table = arch::PageTable::new(mm::page_pool::alloc());
  let entry_point = read_elf(&_binary_user_elf_start, &page_table).unwrap_or(0);
  println!("entry point: {:08x}", entry_point);
  page_table.insert_page(0x8000_0000 - PAGE_SIZE, mm::page_pool::alloc(), PteAttribute::user_default());
  p1.init(1234, entry_point);
  p1.set_page_table(page_table);
  p1.set_status(lib::process::ProcessStatus::Ready);

  p1.sched();

  // kernel mode -> user mode
  ARCH.start_first_process()
}
