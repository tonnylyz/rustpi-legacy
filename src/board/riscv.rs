use core::ops::Range;

pub const BOARD_CORE_NUMBER: usize = 1;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0xc000_0000;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xc000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;
