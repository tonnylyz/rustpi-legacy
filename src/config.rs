use core::ops::Range;

use crate::arch::*;
use crate::board::*;

// non paged memory in kernel (kernel heap memory)
pub const CONFIG_NON_PAGED_MEMORY_SIZE: usize = 0xf00_0000;

// helper functions
pub fn paged_range() -> Range<usize> {
    extern "C" {
        // Note: link-time label, see kernel.aarch64.lds
        fn KERNEL_END();
    }
    let kernel_end = round_up(kva2pa(KERNEL_END as usize), PAGE_SIZE);
    let normal_range = crate::board::Board::normal_memory_range();
    kernel_end..(normal_range.end - CONFIG_NON_PAGED_MEMORY_SIZE)
}

pub fn heap_range() -> Range<usize> {
    let normal_range = crate::board::Board::normal_memory_range();
    (normal_range.end - CONFIG_NON_PAGED_MEMORY_SIZE)..normal_range.end
}

// Process
pub const CONFIG_PROCESS_NUMBER: usize = 1 << 10;
pub const CONFIG_PROCESS_IPC_SIZE: usize = 32;

// aarch64
pub const CONFIG_RECURSIVE_PAGE_TABLE_BTM: usize = 0x3f_c000_0000;

// riscv64
pub const CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_TOP: usize = 0x40_0000_0000; // 1 GB
pub const CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM: usize = 0x3f_c000_0000; // 1 GB
pub const CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000; // 2 MB
pub const CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000 - 0x1000; // 4 KB

pub const CONFIG_USER_IPC_LIST_BTM: usize = 0x3f_a000_0000;
pub const CONFIG_USER_LIMIT: usize = 0x3f_a000_0000;
pub const CONFIG_USER_STACK_TOP: usize = 0x3f_8000_0000;
