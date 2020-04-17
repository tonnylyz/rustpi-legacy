use crate::arch::*;
use crate::driver::mmio::write_byte;
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};

const TIMER_DEFAULT_COUNT: u32 = 10000000;

pub fn next() {
    use cortex_a::regs::*;
    CNTP_TVAL_EL0.set(TIMER_DEFAULT_COUNT);
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE.val(1) + CNTP_CTL_EL0::IMASK.val(0));
}

pub fn init(core_id: usize) {
    if core_id == 0 {
        let page_table = crate::arch::Arch::kernel_page_table();
        page_table.map(0x4000_0000, 0x4000_0000, EntryAttribute::kernel_device());
    }
    unsafe {
        write_byte(0x4000_0040 + core_id * 4, 0b1111);
    }
    next();
}
