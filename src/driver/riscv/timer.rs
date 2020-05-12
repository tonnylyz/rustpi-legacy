use riscv::regs::*;

const SBI_SET_TIMER: usize = 0x00;

const TIMER_DEFAULT_COUNT: usize = 250000;

#[inline(always)]
fn ecall(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
  let ret: usize;
  unsafe {
    llvm_asm!("ecall"
        : "={x10}" (ret)
        : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which)
        : "memory"
        : "volatile");
  }
  ret
}

#[no_mangle]
pub fn next() {
  let time = TIME.get() as usize;
  ecall(SBI_SET_TIMER, time + TIMER_DEFAULT_COUNT, 0, 0);
}

pub fn init(_core_id: usize) {
  next();
  SIE.write(SIE::STIE.val(1));
}
