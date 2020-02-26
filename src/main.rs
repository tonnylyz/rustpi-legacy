#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]

extern crate cortex_a;
extern crate register;
extern crate buddy_system_allocator;
extern crate alloc;

mod driver;
mod lib;

use lib::print;
use lib::vm::vm_init;

#[no_mangle]
#[link_section=".text.start"]
pub unsafe extern "C" fn _start() -> ! {
  use cortex_a::{*,regs::*};
  const CORE_MASK: u64 = 0x3;
  const BOOT_CORE_ID: u64 = 0;
  if BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
    CPACR_EL1.set(3 << 20);
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
    SPSR_EL2.write(
      SPSR_EL2::D::Unmasked
        + SPSR_EL2::A::Unmasked
        + SPSR_EL2::I::Unmasked
        + SPSR_EL2::F::Unmasked
        + SPSR_EL2::M::EL1h,
    );
    ELR_EL2.set(enable_mmu as *const () as u64);
    SP_EL1.set(0x0008_0000);
    asm::eret()
  } else {
    loop {
      asm::wfe()
    }
  }
}

#[no_mangle]
#[link_section=".text.vm"]
pub unsafe fn enable_mmu() -> ! {
  // address over 0x4000_0000 (1GB) will not be mapped.
  // access before MMU enabled
  driver::mmio::mmio_write(0x4000_0040, 0b1111); // timer irq control
  vm_init();
  main();
}

extern "C" {
  static mut _kernel_end: u64;
}

#[no_mangle]
pub unsafe fn main() -> ! {
  driver::uart::uart_init();
  println!("hello");
  lib::exception::set_vbar_el1();
  println!("set_vbar_el1");

  let kernel_end = (((&_kernel_end as *const _ as usize & 0xffff_ffff) >> 12) << 12) + 4096;
  println!("Paged Pool     {:08x}~{:08x}",kernel_end,0x3000_0000);
  println!("Non-Paged Pool {:08x}~{:08x}",0x3000_0000,0x3f00_0000);
  lib::allocator::allocator_init();
  use alloc::boxed::Box;
  let a = Box::new(1);
  println!("box {}", a.as_ref());
  println!("box @{:p}", a.as_ref());
  //driver::timer::timer_init();
  for i in (kernel_end..0x3000_0000).step_by(4096) {
    lib::page_frame::PAGE_FRAMES.push(lib::page_frame::PageFrame::new(i));
  }
  for i in 0..10 {
    let p = lib::page_frame::page_frame_alloc();
    println!("{:p} {:016x}", &p, p.kva());
  }

  loop { }
}

