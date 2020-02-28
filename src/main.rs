#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]

extern crate cortex_a;
extern crate register;
extern crate buddy_system_allocator;
extern crate alloc;
extern crate spin;

mod arch;
mod board;
mod driver;
mod lib;
mod mm;
mod config;

use arch::*;
use lib::*;
use config::*;

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
  let p2 = lib::process::process_alloc();
  p1.init(1);
  p2.init(1000000000000000);

  let page_table = arch::PageTable::new(
    mm::page_pool::alloc()
  );
  extern {
    fn user_program_entry();
  }
  let user_program_frame = mm::PageFrame::new(user_program_entry as usize & 0xffff_ffff);
  page_table.map_frame(0x80000, user_program_frame);

  p1.set_page_table(page_table);
  p2.set_page_table(page_table);
  p1.set_status(lib::process::ProcessStatus::Ready);
  p2.set_status(lib::process::ProcessStatus::Ready);

  p1.sched();

  // kernel mode -> user mode
  ARCH.start_first_process()
}
