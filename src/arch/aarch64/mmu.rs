use super::vm_descriptor::*;
use super::config::*;

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

// Note: this code can be optimized using ARM NEON
// this cannot be run at EL2
// to enable NEON at target config json
// `CPACR_EL1.set(3 << 20); // enable neon over EL1`
// is necessary
#[no_mangle]
#[link_section = ".text.kvm"]
pub unsafe extern "C" fn init() {
  for i in 0..ADDRESS_SPACE_LIMIT_GB {
    let output_addr = TABLES.lvl2[i].base_addr_usize();
    TABLES.lvl1[i] = TableDescriptor::new(output_addr);
    for j in 0..ENTRY_PER_PAGE {
      let output_addr = TABLES.lvl3[i][j].base_addr_usize();
      TABLES.lvl2[i][j] = TableDescriptor::new(output_addr);
      for k in 0..ENTRY_PER_PAGE {
        let output_addr = (i << PAGE_TABLE_L1_SHIFT) | (j << PAGE_TABLE_L2_SHIFT) | (k << PAGE_TABLE_L3_SHIFT);
        TABLES.lvl3[i][j][k] = PageDescriptor::new(output_addr, pa_is_device(output_addr));
      }
    }
  }
  for i in ADDRESS_SPACE_LIMIT_GB..ENTRY_PER_PAGE {
    // avoid optimization using memset (over high address)
    // do NOT write TableDescriptor(0)
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

  TCR_EL1.set(AARCH64_TCR_EL1_VALUE);
  barrier::isb(barrier::SY);
  SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::NonCacheable + SCTLR_EL1::I::NonCacheable);
  barrier::isb(barrier::SY);
}
