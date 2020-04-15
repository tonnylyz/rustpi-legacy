use crate::driver::mmio::*;
// Note: NS16550A uart driver from
// https://github.com/michaeljclark/riscv-probe/blob/master/libfemto/drivers/ns16550a.c

const UART_BASE_ADDR: usize = 0xffff_ffff_0000_0000 + 0x1000_0000;

const UART_RBR      :usize = 0x00;  /* Receive Buffer Register */
const UART_THR      :usize = 0x00;  /* Transmit Hold Register */
const UART_IER      :usize = 0x01;  /* Interrupt Enable Register */
const UART_DLL      :usize = 0x00;  /* Divisor LSB (LCR_DLAB) */
const UART_DLM      :usize = 0x01;  /* Divisor MSB (LCR_DLAB) */
const UART_FCR      :usize = 0x02;  /* FIFO Control Register */
const UART_LCR      :usize = 0x03;  /* Line Control Register */
const UART_MCR      :usize = 0x04;  /* Modem Control Register */
const UART_LSR      :usize = 0x05;  /* Line Status Register */
const UART_MSR      :usize = 0x06;  /* Modem Status Register */
const UART_SCR      :usize = 0x07;  /* Scratch Register */
const UART_LCR_DLAB :u8    = 0x80;  /* Divisor Latch Bit */
const UART_LCR_8BIT :u8    = 0x03;  /* 8-bit */
const UART_LCR_PODD :u8    = 0x08;  /* Parity Odd */
const UART_LSR_DA   :u8    = 0x01;  /* Data Available */
const UART_LSR_OE   :u8    = 0x02;  /* Overrun Error */
const UART_LSR_PE   :u8    = 0x04;  /* Parity Error */
const UART_LSR_FE   :u8    = 0x08;  /* Framing Error */
const UART_LSR_BI   :u8    = 0x10;  /* Break indicator */
const UART_LSR_RE   :u8    = 0x20;  /* THR is empty */
const UART_LSR_RI   :u8    = 0x40;  /* THR is empty and line is idle */
const UART_LSR_EF   :u8    = 0x80;  /* Erroneous data in FIFO */

pub fn init() {
  let base = UART_BASE_ADDR;
  unsafe {
    write_byte(base + UART_FCR, 0);
    write_byte(base + UART_LCR, UART_LCR_DLAB as u8);
    write_byte(base + UART_DLL, (115200 / 9600) as u8);
    write_byte(base + UART_DLM, 0);
    write_byte(base + UART_LCR, UART_LCR_8BIT & !UART_LCR_DLAB);
    write_byte(base + UART_MCR, 0);
  }
}

fn send(c: u8) {
  let base = UART_BASE_ADDR;
  unsafe {
    write_byte(base + UART_THR, c);
  }
}

pub fn putc(c: u8) {
  if c == b'\n' {
    send(b'\r');
  }
  send(c);
}