pub trait InterruptServiceRoutine {
  fn system_call(&self);
  fn interrupt_request(&self);
  fn page_fault(&self);
  fn default(&self);
}

pub struct Isr;

impl InterruptServiceRoutine for Isr {
  fn system_call(&self) {
    println!("InterruptServiceRoutine: system_call");
  }
  fn interrupt_request(&self) {
    println!("InterruptServiceRoutine: interrupt_request");
    crate::driver::timer::timer_next(0);
    super::process::process_schedule();
  }
  fn page_fault(&self) {
    panic!("InterruptServiceRoutine: page_fault");
  }
  fn default(&self) {
    panic!("InterruptServiceRoutine: default");
  }
}

// ARCH code use crate::lib::isr::ISR to invoke kernel function
pub static ISR: Isr = Isr;