#![no_std]
#![no_main]
#![feature(global_asm)]
extern crate cortex_a;
extern crate register;

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

#[no_mangle]
pub unsafe fn main() -> ! {
  driver::uart::uart_init();
  println!("hello");
  lib::exception::set_vbar_el1();
  println!("set_vbar_el1");
  driver::timer::timer_init();
  loop {
    println!("*");
  }
}
