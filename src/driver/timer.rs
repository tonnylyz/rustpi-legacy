use arch::*;
use driver::mmio::mmio_writeb;

const TIMER_DEFAULT_COUNT: u32 = 10000000;

pub fn next(count: u32) {
  use cortex_a::regs::*;
  CNTP_TVAL_EL0.set(if count != 0 { count } else { TIMER_DEFAULT_COUNT });
  CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE.val(1) + CNTP_CTL_EL0::IMASK.val(0));
}

pub fn init() {
  let page_table = ARCH.get_kernel_page_table();
  page_table.map(0x4000_0000, 0x4000_0000, PageTableEntryAttr::kernel_device_default());
  unsafe { mmio_writeb(0x4000_0040, 0b1111); }
  // timer irq control
  next(0);
}
