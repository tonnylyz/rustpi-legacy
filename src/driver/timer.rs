const TIMER_DEFAULT_COUNT : u32 = 10000000;

use cortex_a::regs::*;

pub fn timer_next(count : u32) {
  CNTP_TVAL_EL0.set(if count != 0 { count } else { TIMER_DEFAULT_COUNT });
  CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE.val(1) + CNTP_CTL_EL0::IMASK.val(0));
}

pub fn timer_init() {
  timer_next(0);
}
