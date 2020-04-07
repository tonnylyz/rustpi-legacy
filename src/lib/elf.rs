use crate::{arch, mm};
use arch::{PAGE_SIZE, PageTableImpl, PteAttribute};

unsafe fn memcpy(src: &'static [u8], offset: usize, dest: mm::PageFrame, length: usize) {
  println!("memcpy from {:x} to {:x} len {:x}", offset, dest.pa(), length);
  for i in 0..length {
    *((dest.kva() + i) as *mut u8) = src[offset + i];
  }
}

pub unsafe fn load_elf(src: &'static [u8], page_table: arch::PageTable) -> Result<usize, &'static str>{
  use xmas_elf::*;
  let elf = ElfFile::new(src)?;
  println!("{}", elf.header);
  let entry_point = elf.header.pt2.entry_point() as usize;
  if elf.header.pt2.machine().as_machine() != header::Machine::AArch64 {
    return Err("Unsupported Arch");
  }
  for program_header in elf.program_iter() {
    if program_header.get_type()? != program::Type::Load {
      /* Ignore types other than `Load` */
      continue
    }
    let va = program_header.virtual_addr() as usize;
    let file_size = program_header.file_size() as usize;
    let mem_size = program_header.mem_size() as usize;
    let offset = program_header.offset() as usize;
    for i in (va..va + mem_size).step_by(PAGE_SIZE) {
      /* Note: we require `LOAD` type program data page aligned */
      assert_eq!(i % PAGE_SIZE, 0);
      let frame = mm::page_pool::alloc();
      if i + PAGE_SIZE > va + mem_size {
        // last page
        if i > va + file_size {
          // clean mem exceeded file size
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
      page_table.insert_page(i, frame, PteAttribute::user_default());
    }
  }
  Ok(entry_point)
}