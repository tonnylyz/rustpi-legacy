use cortex_a::regs::RegisterReadWrite;

pub trait InterruptServiceRoutine {
  fn system_call(&self);
  fn interrupt_request(&self);
  fn page_fault(&self);
  fn default(&self);
}

pub struct Isr;

impl InterruptServiceRoutine for Isr {
  fn system_call(&self) {
    use arch::*;
    print!("{}", unsafe { CONTEXT_FRAME }.system_call_argument(0) as u8 as char);
  }
  fn interrupt_request(&self) {
    println!("InterruptServiceRoutine: interrupt_request");
    crate::driver::timer::timer_next(0);
    super::process::process_schedule();
  }
  fn page_fault(&self) {
    use arch::*;
    println!("elr: {:016x}", unsafe { CONTEXT_FRAME }.elr);
    println!("far: {:016x}", cortex_a::regs::FAR_EL1.get());
    panic!("InterruptServiceRoutine: page_fault");
  }
  fn default(&self) {
    panic!("InterruptServiceRoutine: default");
  }
}

// ARCH code use crate::lib::isr::ISR to invoke kernel function
pub static ISR: Isr = Isr;