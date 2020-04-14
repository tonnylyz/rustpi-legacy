use crate::arch::{Arch, ArchTrait};

use crate::driver::mmio::{read_word, write_word};

const GPFSEL1: usize = 0xFFFFFF8000000000 + 0x3F200004;
const GPPUD: usize = 0xFFFFFF8000000000 + 0x3F200094;
const GPPUDCLK0: usize = 0xFFFFFF8000000000 + 0x3F200098;

const AUX_ENABLES: usize = 0xFFFFFF8000000000 + 0x3F215004;
const AUX_MU_IO_REG: usize = 0xFFFFFF8000000000 + 0x3F215040;
const AUX_MU_IER_REG: usize = 0xFFFFFF8000000000 + 0x3F215044;
const AUX_MU_IIR_REG: usize = 0xFFFFFF8000000000 + 0x3F215048;
const AUX_MU_LCR_REG: usize = 0xFFFFFF8000000000 + 0x3F21504C;
const AUX_MU_MCR_REG: usize = 0xFFFFFF8000000000 + 0x3F215050;
const AUX_MU_LSR_REG: usize = 0xFFFFFF8000000000 + 0x3F215054;
const AUX_MU_CNTL_REG: usize = 0xFFFFFF8000000000 + 0x3F215060;
const AUX_MU_BAUD_REG: usize = 0xFFFFFF8000000000 + 0x3F215068;

fn clock_delay(n: u32) -> () {
  for _ in 0..n {
    Arch::nop();
  }
}

pub fn init() -> () {
  unsafe {
    write_word(AUX_ENABLES, 1);
    write_word(AUX_MU_IER_REG, 0);
    write_word(AUX_MU_CNTL_REG, 0);
    write_word(AUX_MU_LCR_REG, 3);
    write_word(AUX_MU_MCR_REG, 0);
    write_word(AUX_MU_IER_REG, 0);
    write_word(AUX_MU_IIR_REG, 0xC6);
    write_word(AUX_MU_BAUD_REG, 270);

    let mut ra = read_word(GPFSEL1);
    ra &= !(7u32 << 12);
    ra |= 2u32 << 12;
    ra &= !(7u32 << 15);
    ra |= 2u32 << 15;
    write_word(GPFSEL1, ra);
    write_word(GPPUD, 0);
    clock_delay(150);
    write_word(GPPUDCLK0, (1u32 << 14) | (1u32 << 15));
    clock_delay(150);
    write_word(GPPUDCLK0, 0);
    write_word(AUX_MU_CNTL_REG, 3);
  }
}

fn send(c: u8) {
  unsafe {
    loop {
      if (read_word(AUX_MU_LSR_REG) & 0x20) != 0 {
        break;
      }
      Arch::nop();
    }
    write_word(AUX_MU_IO_REG, c as u32);
  }
}

pub fn putc(c: u8) {
  if c == b'\n' {
    send(b'\r');
  }
  send(c);
}