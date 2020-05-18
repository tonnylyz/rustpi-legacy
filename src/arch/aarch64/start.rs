use crate::arch::Address;

const CORE_MASK: u64 = 0x3;
const BOOT_CORE_ID: u64 = 0;

#[no_mangle]
#[link_section = ".text.start"]
unsafe extern "C" fn _start() -> ! {
  use cortex_a::{*, regs::*};
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
    SP_EL1.set(0x0008_0000 as u64);
    asm::eret()
  } else {
    loop {
      asm::wfe()
    }
  }
}

#[no_mangle]
#[link_section = ".text.kvm"]
unsafe extern "C" fn el2_start() -> ! {
  use cortex_a::{*, regs::*};
  let core_id = MPIDR_EL1.get() & CORE_MASK;
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
  SP_EL1.set((0x0008_0000 - core_id * 0x0002_0000) as u64);
  asm::eret()
}

#[no_mangle]
#[link_section = ".text.kvm"]
unsafe fn el1_start() -> ! {
  use cortex_a::regs::*;
  let core_id = MPIDR_EL1.get() & CORE_MASK;
  super::mmu::init(core_id == BOOT_CORE_ID);
  SP.set(((0x0008_0000 - core_id * 0x0002_0000) as usize).pa2kva() as u64);
  crate::main();
}
