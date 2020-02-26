#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate cortex_a;
extern crate register;
extern crate buddy_system_allocator;
extern crate alloc;

mod driver;
mod lib;

use lib::print;

#[no_mangle]
#[link_section=".text.start"]
pub unsafe extern "C" fn _start() -> ! {
  use cortex_a::{*,regs::*};
  const CORE_MASK: u64 = 0x3;
  const BOOT_CORE_ID: u64 = 0;
  if BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
    CPACR_EL1.set(3 << 20); // enable neon over EL1
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
    SPSR_EL2.write(
      SPSR_EL2::D::Unmasked
        + SPSR_EL2::A::Unmasked
        + SPSR_EL2::I::Unmasked
        + SPSR_EL2::F::Unmasked
        + SPSR_EL2::M::EL1h,
    );
    ELR_EL2.set(lib::kvm::kvm_enable as *const () as u64);
    SP_EL1.set(0x0008_0000);
    asm::eret()
  } else {
    loop {
      asm::wfe()
    }
  }
}



#[no_mangle]
pub unsafe fn main() -> ! {
  driver::uart::uart_init();
  lib::exception::exception_init();

  extern "C" {
    static mut _kernel_end: u64;
  }
  let kernel_end = 0x300000;//(((&_kernel_end as *const _ as usize & 0xffff_ffff) >> 12) << 12) + 4096;
  println!("Non-Paged Pool {:08x}~{:08x}",0x3000_0000,0x3f00_0000);
  println!("Paged Pool     {:08x}~{:08x}",kernel_end,0x3000_0000);
  lib::allocator::allocator_init(0x3000_0000..0x3f00_0000);
  lib::page_frame::page_frame_init(kernel_end..0x3000_0000);

  driver::timer::timer_init();
  lib::process::process_init();

  let pid = lib::process::process_alloc();
  pid.init_context();
  let upt = lib::uvm::UserPageTable::new(
    lib::page_frame::page_frame_alloc()
  );
  extern {
    fn user_program_entry();
  }
  let user_program_frame = lib::page_frame::PageFrame::new(user_program_entry as usize & 0xffff_ffff);
  upt.map_frame(0x80000, user_program_frame);
  pid.set_page_table(upt);
  pid.sched();

}
