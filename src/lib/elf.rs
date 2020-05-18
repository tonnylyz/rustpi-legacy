use crate::arch::{PAGE_SIZE, PageTable};
use crate::lib::{round_down, round_up};
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::mm::PageFrame;

pub enum Error {
  ElfHeaderError,
  ElfPageTableError,
}

impl core::convert::From<crate::lib::page_table::Error> for Error {
  fn from(_: crate::lib::page_table::Error) -> Self {
    Error::ElfPageTableError
  }
}

#[inline(always)]
fn copy(src: &[u8], src_offset: usize, dst: PageFrame, dst_offset: usize, length: usize) {
  for i in 0..length {
    unsafe {
      ((dst.kva() + dst_offset + i) as *mut u8).write(src[src_offset + i]);
    }
  }
}

pub fn load(src: &'static [u8], page_table: PageTable) -> Result<usize, Error> {
  use xmas_elf::*;
  if let Ok(elf) = ElfFile::new(src) {
    let entry_point = elf.header.pt2.entry_point() as usize;
    for program_header in elf.program_iter() {
      if let Ok(program::Type::Load) = program_header.get_type() {
        /* Ignore types other than `Load` */
      } else {
        continue;
      }
      let va = program_header.virtual_addr() as usize;
      let file_size = program_header.file_size() as usize;
      let file_end = va + file_size;
      let mem_size = program_header.mem_size() as usize;
      let mem_end = va + mem_size;

      let mut offset = program_header.offset() as usize;
      let mut i = va;
      loop {
        if i % PAGE_SIZE != 0 {
          let lo = round_down(i, PAGE_SIZE);
          let hi = round_up(i, PAGE_SIZE);
          let frame = crate::mm::page_pool::alloc();
          frame.zero();
          page_table.insert_page(lo, frame, EntryAttribute::user_default())?;
          if hi < file_end {
            // [lo  i  hi]   file_end    mem_end
            //  ????*****
            copy(src, offset, frame, i - lo, hi - i);
            offset += hi - i;
          } else if file_end <= hi && hi < mem_end {
            // [lo  i     file_end       hi]       mem_end
            //  ????**************000000000
            copy(src, offset, frame, i - lo, file_end - i);
          } else if mem_end <= hi {
            // [lo  i     file_end   mem_end    hi]
            //  ????**************0000000000??????
            copy(src, offset, frame, i - lo, file_end - i);
            break;
          }
          i = hi;
        }

        let lo = i;
        let hi = i + PAGE_SIZE;
        if hi <= file_end {
          // [lo      hi]  file_end   mem_end
          //  **********
          let frame = crate::mm::page_pool::alloc();
          frame.zero();
          page_table.insert_page(lo, frame, EntryAttribute::user_default())?;
          copy(src, offset, frame, 0, PAGE_SIZE);
        } else if lo < file_end && hi < mem_end {
          // [lo   file_end    hi]    mem_end
          //  *************000000
          let frame = crate::mm::page_pool::alloc();
          frame.zero();
          page_table.insert_page(lo, frame, EntryAttribute::user_default())?;
          copy(src, offset, frame, 0, file_end - lo);
        } else if lo < file_end && mem_end <= hi {
          // [lo   file_end    mem_end   hi]
          //  *************00000000000?????
          let frame = crate::mm::page_pool::alloc();
          frame.zero();
          page_table.insert_page(lo, frame, EntryAttribute::user_default())?;
          copy(src, offset, frame, 0, file_end - lo);
          break;
        } else if file_end <= lo && lo < mem_end && mem_end <= hi {
          // file_end  [lo    mem_end   hi]
          //            0000000000000?????
          let frame = crate::mm::page_pool::alloc();
          frame.zero();
          page_table.insert_page(lo, frame, EntryAttribute::user_default())?;
          break;
        } else {
          break;
        }
        offset += PAGE_SIZE;
        i += PAGE_SIZE;
      }
    }
    Ok(entry_point)
  } else {
    Err(Error::ElfHeaderError)
  }
}