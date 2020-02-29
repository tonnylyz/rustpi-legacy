use arch::{ARCH, PtePermission, Arch, PteAttribute, PageTableImpl};
use driver::mmio::mmio_writeb;

const TIMER_DEFAULT_COUNT: u32 = 10000000;

pub fn timer_next(count: u32) {
  use cortex_a::regs::*;
  CNTP_TVAL_EL0.set(if count != 0 { count } else { TIMER_DEFAULT_COUNT });
  CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE.val(1) + CNTP_CTL_EL0::IMASK.val(0));
}

pub fn timer_init() {
  let kpt = ARCH.get_kernel_page_table();
  kpt.map(0x4000_0000, 0x4000_0000, PteAttribute {
    cap_kernel: PtePermission {
      r: true,
      w: true,
      x: false,
    },
    cap_user: PtePermission {
      r: false,
      w: false,
      x: false,
    },
    cow: false,
    library: false,
    device: true,
  });

  unsafe { mmio_writeb(0x4000_0040, 0b1111); }
  // timer irq control
  timer_next(0);
}
