use super::vm_descriptor::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
struct TableDescriptor(u64);

impl TableDescriptor {
  fn new(output_addr: usize) -> TableDescriptor {
    TableDescriptor((
      TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR.val((output_addr >> SHIFT_4KB) as u64)
        + TABLE_DESCRIPTOR::TYPE::Table
        + TABLE_DESCRIPTOR::VALID::True).value)
  }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct PageDescriptor(u64);

impl PageDescriptor {
  fn new(output_addr: usize, device: bool) -> PageDescriptor {
    PageDescriptor((
      PAGE_DESCRIPTOR::PXN::False
        + PAGE_DESCRIPTOR::OUTPUT_ADDR.val((output_addr >> SHIFT_4KB) as u64)
        + PAGE_DESCRIPTOR::AF::True
        + PAGE_DESCRIPTOR::AP::RW_EL1
        + PAGE_DESCRIPTOR::TYPE::Table
        + PAGE_DESCRIPTOR::VALID::True
        +
        if device {
          PAGE_DESCRIPTOR::AttrIndx::DEVICE + PAGE_DESCRIPTOR::SH::OuterShareable
        } else {
          PAGE_DESCRIPTOR::AttrIndx::NORMAL + PAGE_DESCRIPTOR::SH::InnerShareable
        }
    ).value)
  }
}

const ADDRESS_SPACE_LIMIT_GB: usize = 1;

const SHIFT_4KB: usize = 12;
const PAGE_SIZE: usize = 1 << SHIFT_4KB;
const ENTRY_PER_PAGE: usize = PAGE_SIZE / 8;

#[repr(C)]
#[repr(align(4096))]
struct PageTables {
  lvl3: [[[PageDescriptor; ENTRY_PER_PAGE]; ENTRY_PER_PAGE]; ADDRESS_SPACE_LIMIT_GB],
  lvl2: [[TableDescriptor; ENTRY_PER_PAGE]; ADDRESS_SPACE_LIMIT_GB],
  lvl1: [TableDescriptor; ENTRY_PER_PAGE],
}

#[no_mangle]
#[link_section = ".data.kvm"]
static mut TABLES: PageTables = PageTables {
  lvl3: [[[PageDescriptor(0); ENTRY_PER_PAGE]; ENTRY_PER_PAGE]; ADDRESS_SPACE_LIMIT_GB],
  lvl2: [[TableDescriptor(0); ENTRY_PER_PAGE]; ADDRESS_SPACE_LIMIT_GB],
  lvl1: [TableDescriptor(0); ENTRY_PER_PAGE],
};

trait BaseAddr {
  fn base_addr_u64(&self) -> u64;
  fn base_addr_usize(&self) -> usize;
}

impl<T> BaseAddr for T {
  fn base_addr_u64(&self) -> u64 {
    self as *const T as u64
  }
  fn base_addr_usize(&self) -> usize {
    self as *const T as usize
  }
}

use cortex_a::regs::*;
use cortex_a::*;

#[no_mangle]
#[link_section = ".text.kvm"]
unsafe extern "C" fn kvm_init() {
  for i in 0..ADDRESS_SPACE_LIMIT_GB {
    let output_addr = TABLES.lvl2[i].base_addr_usize();
    TABLES.lvl1[i] = TableDescriptor::new(output_addr);
    for j in 0..ENTRY_PER_PAGE {
      let output_addr = TABLES.lvl3[i][j].base_addr_usize();
      TABLES.lvl2[i][j] = TableDescriptor::new(output_addr);
      for k in 0..ENTRY_PER_PAGE {
        let output_addr = (i << 30) | (j << 21) | (k << 12);
        TABLES.lvl3[i][j][k] = PageDescriptor::new(output_addr, output_addr >= 0x3f00_0000);
      }
    }
  }
  for i in ADDRESS_SPACE_LIMIT_GB..ENTRY_PER_PAGE {
    // avoid optimization using memset (over high address)
    TABLES.lvl1[i] = TableDescriptor((i << 2) as u64);
  }
  MAIR_EL1.write(
    MAIR_EL1::Attr0_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc
      + MAIR_EL1::Attr0_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc
      + MAIR_EL1::Attr1_HIGH::Device
      + MAIR_EL1::Attr1_LOW_DEVICE::Device_nGnRE,
  );
  TTBR0_EL1.set(TABLES.lvl1.base_addr_u64());
  TTBR1_EL1.set(TABLES.lvl1.base_addr_u64());
  // Avoid register code using over high address
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
  TCR_EL1.set(0x0000006135193519);
  barrier::isb(barrier::SY);
  SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
  barrier::isb(barrier::SY);
}

#[no_mangle]
#[link_section = ".text.kvm"]
pub unsafe fn kvm_enable() -> ! {
  use cortex_a::regs::*;
  // address over 0x4000_0000 (1GB) will not be mapped.
  // access before MMU enabled
  crate::driver::mmio::mmio_write(0x4000_0040, 0b1111); // timer irq control
  kvm_init();
  SP.set(0xFFFFFF8000000000 + 0x0008_0000);
  crate::main();
}