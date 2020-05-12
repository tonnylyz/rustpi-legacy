use crate::arch::{PAGE_SIZE, PageTable};
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::mm::PageFrame;

unsafe fn memcpy(src: &'static [u8], offset: usize, dest: PageFrame, length: usize) {
  //println!("memcpy from {:x} to {:x} len {:x}", offset, dest.pa(), length);
  for i in 0..length {
    *((dest.kva() + i) as *mut u8) = src[offset + i];
  }
}

// TODO: (elf loader) make a robuster elf loader
pub unsafe fn load_elf(src: &'static [u8], page_table: PageTable) -> usize {
  use xmas_elf::*;
  if let Ok(elf) = ElfFile::new(src) {
    //println!("{}", elf.header);
    let entry_point = elf.header.pt2.entry_point() as usize;
    for program_header in elf.program_iter() {
      if let Ok(program::Type::Load) = program_header.get_type() {
        /* Ignore types other than `Load` */
      } else {
        continue;
      }
      let va = program_header.virtual_addr() as usize;
      let file_size = program_header.file_size() as usize;
      let mem_size = program_header.mem_size() as usize;
      let offset = program_header.offset() as usize;
      for i in (va..va + mem_size).step_by(PAGE_SIZE) {
        /* Note: we require `LOAD` type program data page aligned */
        assert_eq!(i % PAGE_SIZE, 0);
        let frame = crate::mm::page_pool::alloc();
        if i + PAGE_SIZE > va + mem_size {
          // last page
          if i > va + file_size {
            frame.zero();
          } else {
            memcpy(src, offset + i - va, frame, va + file_size - i);
          }
        } else {
          if i > va + file_size {
            frame.zero();
          } else {
            memcpy(src, offset + i - va, frame, PAGE_SIZE);
          }
        }
        //for i in (0..PAGE_SIZE).step_by(4) {
        //  print!("{:08x}", *((i + frame.kva()) as *const u32));
        //  if (i + 1) % (8 * 2) == 0 {
        //    println!();
        //  }
        //}
        match page_table.insert_page(i, frame, EntryAttribute::user_default()) {
          Ok(_) => {}
          Err(_) => { panic!("elf: page_table.insert_page") }
        }
      }
    }
    entry_point
  } else {
    0
  }
}