use crate::board::*;

use super::interface::PAGE_SHIFT;
use super::interface::PAGE_SIZE;
use super::page_table::PAGE_TABLE_L1_SHIFT;
use super::page_table::PAGE_TABLE_L2_SHIFT;
use super::page_table::PAGE_TABLE_L3_SHIFT;
use super::vm_descriptor::*;

const PHYSICAL_ADDRESS_LIMIT_GB: usize = BOARD_PHYSICAL_ADDRESS_LIMIT >> 30;
const ENTRY_PER_PAGE: usize = PAGE_SIZE / 8;

#[derive(Eq, PartialEq)]
enum MemoryType {
  Normal,
  Device,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct TableDescriptor(u64);

impl TableDescriptor {
  fn new(output_addr: usize) -> TableDescriptor {
    TableDescriptor((
      TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_PPN.val((output_addr >> PAGE_SHIFT) as u64)
        + TABLE_DESCRIPTOR::TYPE::Table
        + TABLE_DESCRIPTOR::VALID::True).value)
  }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct PageDescriptor(u64);

impl PageDescriptor {
  fn new(output_addr: usize, t: MemoryType) -> PageDescriptor {
    PageDescriptor((
      PAGE_DESCRIPTOR::PXN::False
        + PAGE_DESCRIPTOR::OUTPUT_PPN.val((output_addr >> PAGE_SHIFT) as u64)
        + PAGE_DESCRIPTOR::AF::True
        + PAGE_DESCRIPTOR::AP::RW_EL1
        + PAGE_DESCRIPTOR::TYPE::Table
        + PAGE_DESCRIPTOR::VALID::True
        +
        if t == MemoryType::Device {
          PAGE_DESCRIPTOR::AttrIndx::DEVICE + PAGE_DESCRIPTOR::SH::OuterShareable
        } else { // if t == MemoryType::Normal {
          PAGE_DESCRIPTOR::AttrIndx::NORMAL + PAGE_DESCRIPTOR::SH::InnerShareable
        }// else { panic!("Undefined memory type") }
    ).value)
  }
}

#[repr(C)]
#[repr(align(4096))]
struct PageTables {
  lvl3: [[[PageDescriptor; ENTRY_PER_PAGE]; ENTRY_PER_PAGE]; PHYSICAL_ADDRESS_LIMIT_GB],
  lvl2: [[TableDescriptor; ENTRY_PER_PAGE]; PHYSICAL_ADDRESS_LIMIT_GB],
  lvl1: [TableDescriptor; ENTRY_PER_PAGE],
}

#[no_mangle]
#[link_section = ".data.kvm"]
static mut KERNEL_PAGE_TABLES: PageTables = PageTables {
  lvl3: [[[PageDescriptor(0); ENTRY_PER_PAGE]; ENTRY_PER_PAGE]; PHYSICAL_ADDRESS_LIMIT_GB],
  lvl2: [[TableDescriptor(0); ENTRY_PER_PAGE]; PHYSICAL_ADDRESS_LIMIT_GB],
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


// Note: this code can be optimized using ARM NEON
// this cannot be run at EL2
// to enable NEON at target config json
// `CPACR_EL1.set(3 << 20); // enable neon over EL1`
// is necessary
#[no_mangle]
#[link_section = ".text.kvm"]
pub unsafe extern "C" fn init(create_table: bool) {
  use cortex_a::regs::*;
  use cortex_a::*;
  if create_table {
    for i in 0..PHYSICAL_ADDRESS_LIMIT_GB {
      let output_addr = KERNEL_PAGE_TABLES.lvl2[i].base_addr_usize();
      KERNEL_PAGE_TABLES.lvl1[i] = TableDescriptor::new(output_addr);
      for j in 0..ENTRY_PER_PAGE {
        let output_addr = KERNEL_PAGE_TABLES.lvl3[i][j].base_addr_usize();
        KERNEL_PAGE_TABLES.lvl2[i][j] = TableDescriptor::new(output_addr);
        for k in 0..ENTRY_PER_PAGE {
          let output_addr = (i << PAGE_TABLE_L1_SHIFT) | (j << PAGE_TABLE_L2_SHIFT) | (k << PAGE_TABLE_L3_SHIFT);
          if crate::board::BOARD_NORMAL_MEMORY_RANGE.contains(&output_addr) {
            KERNEL_PAGE_TABLES.lvl3[i][j][k] = PageDescriptor::new(output_addr, MemoryType::Normal);
          } else if crate::board::BOARD_DEVICE_MEMORY_RANGE.contains(&output_addr) {
            KERNEL_PAGE_TABLES.lvl3[i][j][k] = PageDescriptor::new(output_addr, MemoryType::Device);
          }
        }
      }
    }
    for i in PHYSICAL_ADDRESS_LIMIT_GB..ENTRY_PER_PAGE {
      // avoid optimization using memset (over high address)
      // do NOT write TableDescriptor(0)
      KERNEL_PAGE_TABLES.lvl1[i] = TableDescriptor((i << 4) as u64);
    }
  }

  MAIR_EL1.write(
    MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
      + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
      + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_EarlyWriteAck
  );
  TTBR0_EL1.set(KERNEL_PAGE_TABLES.lvl1.base_addr_u64());
  TTBR1_EL1.set(KERNEL_PAGE_TABLES.lvl1.base_addr_u64());

  TCR_EL1.write(TCR_EL1::TBI0::Ignored
    + TCR_EL1::TBI1::Ignored
    + TCR_EL1::IPS.val(0b001) // 64GB
    + TCR_EL1::TG0::KiB_4
    + TCR_EL1::TG1::KiB_4
    + TCR_EL1::SH0::Inner
    + TCR_EL1::SH1::Inner
    + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
    + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
    + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
    + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
    + TCR_EL1::EPD0::EnableTTBR0Walks
    + TCR_EL1::EPD1::EnableTTBR1Walks
    + TCR_EL1::T0SZ.val(64 - 39)
    + TCR_EL1::T1SZ.val(64 - 39));
  //extern "C" {
  //  fn smpen();
  //}
  //smpen();
  barrier::isb(barrier::SY);
  SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
  barrier::isb(barrier::SY);
}
