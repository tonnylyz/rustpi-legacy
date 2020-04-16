use crate::main;
use riscv::{asm::*, regs::*};

global_asm!(include_str!("start.S"));

// Workaround for abort symbol not found
#[no_mangle]
pub extern "C" fn abort() {
  panic!("abort");
}
