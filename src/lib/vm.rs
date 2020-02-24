use register::*;

// A table descriptor, as per AArch64 Reference Manual Figure D4-15.
register_bitfields! {u64,
    TABLE_DESCRIPTOR [
        /// Physical address of the next page table.
        NEXT_LEVEL_TABLE_ADDR OFFSET(16) NUMBITS(32) [], // [47:16]

        TYPE  OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

// A level 3 page descriptor, as per AArch64 Reference Manual Figure D4-17.
register_bitfields! {u64,
    PAGE_DESCRIPTOR [
        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next page table (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR OFFSET(16) NUMBITS(32) [], // [47:16]

        /// Access flag.
        AF       OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Shareability field.
        SH       OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

        /// Access Permissions.
        AP       OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11
        ],

        /// Memory attributes index into the MAIR_EL1 register.
        AttrIndx OFFSET(2) NUMBITS(3) [],

        TYPE     OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID    OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

const SIXTYFOUR_KIB_SHIFT: usize = 16; //  log2(64 * 1024)
const FIVETWELVE_MIB_SHIFT: usize = 29; // log2(512 * 1024 * 1024)

#[derive(Copy, Clone)]
#[repr(transparent)]
struct TableDescriptor(u64);

impl TableDescriptor {
  fn new(output_addr: usize) -> TableDescriptor {
    let shifted = output_addr >> SIXTYFOUR_KIB_SHIFT;
    let val = (
      TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR.val(shifted as u64)
        + TABLE_DESCRIPTOR::TYPE::Table
        + TABLE_DESCRIPTOR::VALID::True).value;
    TableDescriptor(val)
  }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct PageDescriptor(u64);

impl PageDescriptor {
  fn new(output_addr: usize, device : bool) -> PageDescriptor {
    let shifted = output_addr >> SIXTYFOUR_KIB_SHIFT;
    let val = (
      PAGE_DESCRIPTOR::PXN::False
        + PAGE_DESCRIPTOR::OUTPUT_ADDR.val(shifted as u64)
        + PAGE_DESCRIPTOR::AF::True
        + (if device {PAGE_DESCRIPTOR::SH::OuterShareable} else {PAGE_DESCRIPTOR::SH::InnerShareable})
        + PAGE_DESCRIPTOR::AP::RW_EL1
        + PAGE_DESCRIPTOR::AttrIndx.val((if device {0} else {1}) as u64)
        + PAGE_DESCRIPTOR::TYPE::Table
        + PAGE_DESCRIPTOR::VALID::True
    ).value;
    PageDescriptor(val)
  }
}

/// Big monolithic struct for storing the page tables. Individual levels must be 64 KiB aligned,
/// hence the "reverse" order of appearance.
#[repr(C)]
#[repr(align(65536))]
struct PageTables<const N: usize> {
  // Page descriptors, covering 64 KiB windows per entry.
  lvl3: [[PageDescriptor; 8192]; N],
  // Table descriptors, covering 512 MiB windows.
  lvl2: [TableDescriptor; N],
}

/// Usually evaluates to 1 GiB for RPi3 and 4 GiB for RPi 4.
const ENTRIES_512_MIB: usize = 0x4000_0000 >> FIVETWELVE_MIB_SHIFT;

/// The page tables.
///
/// Supposed to land in `.bss`. Therefore, ensure that they boil down to all "0" entries.
static mut TABLES: PageTables<{ ENTRIES_512_MIB }> = PageTables {
  lvl3: [[PageDescriptor(0); 8192]; ENTRIES_512_MIB],
  lvl2: [TableDescriptor(0); ENTRIES_512_MIB],
};

trait BaseAddr {
  fn base_addr_u64(&self) -> u64;
  fn base_addr_usize(&self) -> usize;
}

impl<T, const N: usize> BaseAddr for [T; N] {
  fn base_addr_u64(&self) -> u64 {
    self as *const T as u64
  }

  fn base_addr_usize(&self) -> usize {
    self as *const T as usize
  }
}


use cortex_a::regs::*;
use cortex_a::*;

pub unsafe fn vm_init() {
  for (l2_nr, l2_entry) in TABLES.lvl2.iter_mut().enumerate() {
    *l2_entry = TableDescriptor::new(TABLES.lvl3[l2_nr].base_addr_usize());

    for (l3_nr, l3_entry) in TABLES.lvl3[l2_nr].iter_mut().enumerate() {
      let virt_addr = (l2_nr << FIVETWELVE_MIB_SHIFT) + (l3_nr << SIXTYFOUR_KIB_SHIFT);
      *l3_entry = PageDescriptor::new(virt_addr, virt_addr >= 0x3f00_0000);
    }
  }
  // Define the memory types being mapped.
  MAIR_EL1.write(
    // Attribute 1 - Cacheable normal DRAM.
    MAIR_EL1::Attr1_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc
      + MAIR_EL1::Attr1_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc

      // Attribute 0 - Device.
      + MAIR_EL1::Attr0_HIGH::Device
      + MAIR_EL1::Attr0_LOW_DEVICE::Device_nGnRE,
  );
  TTBR0_EL1.set_baddr(TABLES.lvl2.base_addr_u64());
  let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);
  TCR_EL1.write(
    TCR_EL1::TBI0::Ignored
      + TCR_EL1::IPS.val(ips)
      + TCR_EL1::TG0::KiB_64
      + TCR_EL1::SH0::Inner
      + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      + TCR_EL1::EPD0::EnableTTBR0Walks
      + TCR_EL1::T0SZ.val(32), // TTBR0 spans 4 GiB total.
  );
  barrier::isb(barrier::SY);
  SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
  barrier::isb(barrier::SY);
}