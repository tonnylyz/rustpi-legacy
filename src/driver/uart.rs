use super::mmio::{mmio_read, mmio_write};

const GPFSEL1: usize = 0x3F200004;
const GPPUD: usize = 0x3F200094;
const GPPUDCLK0: usize = 0x3F200098;

const AUX_ENABLES: usize = 0x3F215004;
const AUX_MU_IO_REG: usize = 0x3F215040;
const AUX_MU_IER_REG: usize = 0x3F215044;
const AUX_MU_IIR_REG: usize = 0x3F215048;
const AUX_MU_LCR_REG: usize = 0x3F21504C;
const AUX_MU_MCR_REG: usize = 0x3F215050;
const AUX_MU_LSR_REG: usize = 0x3F215054;
const AUX_MU_CNTL_REG: usize = 0x3F215060;
const AUX_MU_BAUD_REG: usize = 0x3F215068;

fn clock_delay(n: u32) -> () {
  for _ in 0..n {
    cortex_a::asm::nop();
  }
}

pub fn uart_init() -> () {
  unsafe {
    mmio_write(AUX_ENABLES, 1);
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_CNTL_REG, 0);
    mmio_write(AUX_MU_LCR_REG, 3);
    mmio_write(AUX_MU_MCR_REG, 0);
    mmio_write(AUX_MU_IER_REG, 0);
    mmio_write(AUX_MU_IIR_REG, 0xC6);
    mmio_write(AUX_MU_BAUD_REG, 270);

    let mut ra = mmio_read(GPFSEL1);
    ra &= !(7u32 << 12);
    ra |= 2u32 << 12;
    ra &= !(7u32 << 15);
    ra |= 2u32 << 15;
    mmio_write(GPFSEL1, ra);
    mmio_write(GPPUD, 0);
    clock_delay(150);
    mmio_write(GPPUDCLK0, (1u32 << 14) | (1u32 << 15));
    clock_delay(150);
    mmio_write(GPPUDCLK0, 0);
    mmio_write(AUX_MU_CNTL_REG, 3);
  }
}

fn uart_send(c: u8) {
  unsafe {
    loop {
      if (mmio_read(AUX_MU_LSR_REG) & 0x20) != 0 {
        break;
      }
      cortex_a::asm::nop();
    }
    mmio_write(AUX_MU_IO_REG, c as u32);
  }
}

pub fn uart_putc(c: u8) {
  if c == b'\n' {
    uart_send(b'\r');
  }
  uart_send(c);
}