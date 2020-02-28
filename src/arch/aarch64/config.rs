pub const ADDRESS_SPACE_LIMIT_GB: usize = 1;
pub const SHIFT_4KB: usize = 12;
pub const PAGE_SIZE: usize = 1 << SHIFT_4KB;
pub const ENTRY_PER_PAGE: usize = PAGE_SIZE / 8;

pub const KERNEL_STACKTOP_PA: usize = 0x0008_0000;

pub const KERNEL_ADDRESS_START: usize = 0xFFFFFF8000000000;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

// Note: workaround for kernel low address code constrain
//let tcr = (TCR_EL1::TBI0::Ignored
//  + TCR_EL1::TBI1::Ignored
//  + TCR_EL1::IPS.val(0b001) // 64GB
//  + TCR_EL1::TG0::KiB_4
//  + TCR_EL1::TG1::KiB_4
//  + TCR_EL1::SH0::Inner
//  + TCR_EL1::SH1::Inner
//  + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
//  + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
//  + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
//  + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
//  + TCR_EL1::EPD0::EnableTTBR0Walks
//  + TCR_EL1::EPD1::EnableTTBR1Walks
//  + TCR_EL1::T0SZ.val(64 - 39)
//  + TCR_EL1::T1SZ.val(64 - 39)).value;
pub const AARCH64_TCR_EL1_VALUE: u64 = 0x0000006135193519;

// TODO: device specified function
#[inline(always)]
pub fn pa_is_device(pa : usize) -> bool {
  pa >= 0x3f00_0000
}


// helper function
#[inline(always)]
pub fn pa2kva(pa : usize) -> usize {
  KERNEL_ADDRESS_START | pa
}
