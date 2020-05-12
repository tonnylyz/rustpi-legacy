use crate::arch::ArchTrait;
use crate::driver::mmio::*;

// platform level interrupt controller
const PLIC_BASE_ADDR: usize = 0xffff_ffff_0000_0000 + 0x0c00_0000;
const PLIC_PENDING_ADDR: usize = PLIC_BASE_ADDR + 0x1000;
const PLIC_MACHINE_ENABLE_ADDR: usize = PLIC_BASE_ADDR + 0x2000;
const PLIC_MACHINE_PRIORITY_ADDR: usize = PLIC_BASE_ADDR + 0x200000;
const PLIC_MACHINE_CLAIM_ADDR: usize = PLIC_BASE_ADDR + 0x200004;

const PLIC_SUPERVISOR_ENABLE_ADDR: usize = PLIC_BASE_ADDR + 0x2080;
// by 0x100
const PLIC_SUPERVISOR_PRIORITY_ADDR: usize = PLIC_BASE_ADDR + 0x201000;
// by 0x2000
const PLIC_SUPERVISOR_CLAIM_ADDR: usize = PLIC_BASE_ADDR + 0x201004;
// by 0x2000

const PLIC_IRQ_VIRTIO: usize = 1;
const PLIC_IRQ_UART: usize = 10;

pub fn init() {
  unsafe {
    write_word(PLIC_BASE_ADDR + PLIC_IRQ_UART * 4, 1);
    write_word(PLIC_BASE_ADDR + PLIC_IRQ_VIRTIO * 4, 1);
    let core_id = crate::arch::Arch::core_id();
    write_word(PLIC_SUPERVISOR_ENABLE_ADDR + core_id * 0x100, ((1 << PLIC_IRQ_VIRTIO) | (1 << PLIC_IRQ_UART)) as u32);
    write_word(PLIC_SUPERVISOR_PRIORITY_ADDR + core_id * 0x2000, 0);
  }
}

pub fn pending() -> usize {
  unsafe {
    let mask = read_dword(PLIC_PENDING_ADDR);
    mask as usize
  }
}

pub fn claim() -> usize {
  unsafe {
    let core_id = crate::arch::Arch::core_id();
    read_word(PLIC_MACHINE_CLAIM_ADDR + core_id * 0x2000) as usize
  }
}

pub fn clear(irq: usize) {
  unsafe {
    let core_id = crate::arch::Arch::core_id();
    write_word(PLIC_MACHINE_CLAIM_ADDR + core_id * 0x2000, irq as u32);
  }
}
