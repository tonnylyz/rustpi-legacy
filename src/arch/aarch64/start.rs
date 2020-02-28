use super::config::*;

#[no_mangle]
#[link_section = ".text.start"]
unsafe extern "C" fn _start() -> ! {
  use cortex_a::{*, regs::*};
  const CORE_MASK: u64 = 0x3;
  const BOOT_CORE_ID: u64 = 0;
  if BOOT_CORE_ID == MPIDR_EL1.get() & CORE_MASK {
    CPACR_EL1.set(3 << 20); // enable neon over EL1
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
    SPSR_EL2.write(
      SPSR_EL2::D::Unmasked
        + SPSR_EL2::A::Unmasked
        + SPSR_EL2::I::Masked // mask irq and fiq
        + SPSR_EL2::F::Masked
        + SPSR_EL2::M::EL1h,
    );
    ELR_EL2.set(el1_start as *const () as u64);
    SP_EL1.set(KERNEL_STACKTOP_PA as u64);
    asm::eret()
  } else {
    loop {
      asm::wfe()
    }
  }
}

#[no_mangle]
#[link_section = ".text.kvm"]
unsafe fn el1_start() -> ! {
  use cortex_a::regs::*;
  // TODO: move this out
  // address over 0x4000_0000 (1GB) will not be mapped.
  // access before MMU enabled
  core::intrinsics::volatile_store(0x4000_0040 as *mut u8, 0b1111); // timer irq control
  super::mmu::init();
  SP.set(pa2kva(KERNEL_STACKTOP_PA) as u64);
  crate::main();
}
