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

mod arch;
mod driver;
mod lib;

use arch::*;
use lib::*;

#[no_mangle]
pub unsafe fn main() -> ! {
  driver::uart::uart_init();
  ARCH.exception_init();

  extern "C" {
    // Note: link-time label, see kernel.lds
    static mut _kernel_end: u64;
  }
  let kernel_end = 0x300000;//(((&_kernel_end as *const _ as usize & 0xffff_ffff) >> 12) << 12) + 4096;
  println!("Non-Paged Pool {:08x}~{:08x}", 0x3000_0000, 0x3f00_0000);
  println!("Paged Pool     {:08x}~{:08x}", kernel_end, 0x3000_0000);
  allocator::init(0x3000_0000..0x3f00_0000);
  page_frame::init(kernel_end..0x3000_0000);

  driver::timer::timer_init();

  let p1 = lib::process::process_alloc();
  let p2 = lib::process::process_alloc();
  p1.init(1);
  p2.init(1000000000000000);

  let page_table = lib::uvm::PageTable::new(
    lib::page_frame::page_frame_alloc()
  );
  extern {
    fn user_program_entry();
  }
  let user_program_frame = lib::page_frame::PageFrame::new(user_program_entry as usize & 0xffff_ffff);
  page_table.map_frame(0x80000, user_program_frame);

  p1.set_page_table(page_table);
  p2.set_page_table(page_table);
  p1.set_status(lib::process::ProcessStatus::Ready);
  p2.set_status(lib::process::ProcessStatus::Ready);

  p1.sched();

  // el1 -> el0
  ARCH.start_first_process()
}
