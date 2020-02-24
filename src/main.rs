#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(const_generics)]
extern crate cortex_a;
extern crate register;

mod driver;
mod lib;

use lib::print;
use driver::mmio::mmio_read;

#[no_mangle]
#[link_section=".text.start"]
pub unsafe extern "C" fn _start() -> ! {
  use cortex_a::{*,regs::*};
  const CORE_MASK: u64 = 0x3;
  const BOOT_CORE_ID: u64 = 0;
  if BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
    // MRS X1, CPACR_EL1
    // ORR X1, X1, #(0x3 << 20)
    // MSR CPACR_EL1, X1
    // ISB
    CPACR_EL1.set(3 << 20);
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
    SPSR_EL2.write(
      SPSR_EL2::D::Unmasked
        + SPSR_EL2::A::Unmasked
        + SPSR_EL2::I::Unmasked
        + SPSR_EL2::F::Unmasked
        + SPSR_EL2::M::EL1h,
    );
    ELR_EL2.set(main as *const () as u64);
    SP_EL1.set(0x80000);
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
  driver::mmio::mmio_write(0x4000_0040, 0b1111); // timer int control
  println!("hello");
  lib::vm::vm_init();
  lib::exception::set_vbar_el1();
  println!("set_vbar_el1");
  driver::timer::timer_init();
  loop {
    println!("*");
  }
}
