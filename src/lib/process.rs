use lib::page_frame::PageFrame;
use lib::uvm::UserPageTable;

// Process Control Block
pub struct Process {
  pid: u32,
  page_table: UserPageTable,
  context: super::exception::TrapFrame,
  entry: u64,
}

global_asm!(include_str!("program.S"));

impl Process {
  pub fn new() -> Self {
    extern {
      fn user_program_entry();
      fn pop_time_stack();
    }
    let frame = super::page_frame::page_frame_alloc();
    let upt = super::uvm::UserPageTable::new(frame);
    let text_frame = super::page_frame::PageFrame::new(user_program_entry as usize & 0xffff_ffff);
    upt.map_frame(0x80000, &text_frame);
    upt.install();
    unsafe {
      use cortex_a::regs::*;
      use cortex_a::asm::*;
      ELR_EL1.set(0x80000);
      SPSR_EL1.set(0x80000);
      SP_EL0.set(0x80000000);
      eret();
    }
    Process {
      pid: 0,
      page_table: upt,
      context: super::exception::TrapFrame::default(),
      entry: 0x80000,
    }
  }
}