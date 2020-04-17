use crate::arch::*;
use crate::config::*;
use crate::lib::page_table::{Entry, EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::lib::process::{Pid, Process};
use crate::mm::PageFrame;

use self::Error::*;

pub enum Error {
    InvalidArgumentError = 1,
    OutOfProcessError,
    OutOfMemoryError,
    ProcessPidNotFoundError,
    ProcessParentNotFoundError,
    ProcessParentMismatchedError,
    MemoryLimitError,
    MemoryNotMappedError,
    IpcNotReceivingError,
    InternalError,
}

impl core::convert::From<crate::mm::page_pool::Error> for Error {
    fn from(e: crate::mm::page_pool::Error) -> Self {
        match e {
            crate::mm::page_pool::Error::OutOfFrameError => OutOfMemoryError,
            _ => InternalError,
        }
    }
}

impl core::convert::From<crate::lib::page_table::Error> for Error {
    fn from(_: crate::lib::page_table::Error) -> Self {
        InternalError
    }
}

impl core::convert::From<crate::lib::process_pool::Error> for Error {
    fn from(e: crate::lib::process_pool::Error) -> Self {
        match e {
            crate::lib::process_pool::Error::OutOfProcessError => OutOfProcessError,
            _ => InternalError,
        }
    }
}

pub trait SystemCallTrait {
    fn putc(c: char);
    fn getpid() -> u16;
    fn process_yield();
    fn process_destroy(pid: u16) -> Result<(), Error>;
    fn process_set_exception_handler(pid: u16, value: usize, sp: usize) -> Result<(), Error>;
    fn mem_alloc(pid: u16, va: usize, perm: usize) -> Result<(), Error>;
    fn mem_map(
        src_pid: u16,
        src_va: usize,
        dst_pid: u16,
        dst_va: usize,
        perm: usize,
    ) -> Result<(), Error>;
    fn mem_unmap(pid: u16, va: usize) -> Result<(), Error>;
    fn process_alloc() -> Result<u16, Error>;
    fn process_set_status(pid: u16, status: super::process::Status) -> Result<(), Error>;
    fn ipc_receive(dst_va: usize);
    fn ipc_can_send(pid: u16, value: usize, src_va: usize, perm: usize) -> Result<(), Error>;
}

pub struct SystemCall;

fn lookup_pid(pid: u16, check_parent: bool) -> Result<Process, Error> {
    use crate::lib::process_pool::*;
    if pid == 0 {
        Ok(crate::arch::Arch::running_process().ok_or_else(|| InternalError)?)
    } else {
        if let Some(p) = lookup(Process::new(pid)) {
            if check_parent {
                if let Some(parent) = p.parent() {
                    if crate::arch::Arch::running_process()
                        .ok_or_else(|| InternalError)?
                        .pid()
                        == parent.pid()
                    {
                        Ok(p)
                    } else {
                        Err(ProcessParentMismatchedError)
                    }
                } else {
                    Err(ProcessParentNotFoundError)
                }
            } else {
                Ok(p)
            }
        } else {
            Err(ProcessPidNotFoundError)
        }
    }
}

impl SystemCallTrait for SystemCall {
    fn putc(c: char) {
        crate::driver::uart::putc(c as u8);
    }

    fn getpid() -> u16 {
        crate::arch::Arch::running_process().unwrap().pid()
    }

    fn process_yield() {
        crate::lib::scheduler::schedule();
    }

    fn process_destroy(pid: u16) -> Result<(), Error> {
        let p = lookup_pid(pid, true)?;
        p.destroy();
        Ok(())
    }

    fn process_set_exception_handler(pid: u16, value: usize, sp: usize) -> Result<(), Error> {
        let p = lookup_pid(pid, true)?;
        if value >= CONFIG_USER_LIMIT || sp >= CONFIG_USER_LIMIT || sp % PAGE_SIZE != 0 {
            return Err(InvalidArgumentError);
        }
        unsafe {
            (*p.pcb()).exception_handler = value;
            (*p.pcb()).exception_stack_top = sp;
        }
        Ok(())
    }

    fn mem_alloc(pid: u16, va: usize, attr: usize) -> Result<(), Error> {
        if va >= CONFIG_USER_LIMIT {
            return Err(MemoryLimitError);
        }
        let p = lookup_pid(pid, true)?;
        let frame = crate::mm::page_pool::try_alloc()?;
        frame.zero();
        unsafe {
            let page_table = (*p.pcb()).page_table.ok_or_else(|| InternalError)?;
            let user_attr = Entry::from(ArchPageTableEntry::new(attr)).attribute();
            let attr = user_attr.filter();
            //println!("mem alloc [{:016x}] [{:?}]", va, attr);
            page_table.insert_page(va, frame, attr)?;
        }
        Ok(())
    }

    fn mem_map(
        src_pid: u16,
        src_va: usize,
        dst_pid: u16,
        dst_va: usize,
        attr: usize,
    ) -> Result<(), Error> {
        let src_va = round_down(src_va, PAGE_SIZE);
        let dst_va = round_down(dst_va, PAGE_SIZE);
        if src_va >= CONFIG_USER_LIMIT || dst_va >= CONFIG_USER_LIMIT {
            return Err(MemoryLimitError);
        }
        let src_pid = lookup_pid(src_pid, true)?;
        let dst_pid = lookup_pid(dst_pid, true)?;
        unsafe {
            let src_pt = (*src_pid.pcb()).page_table.ok_or_else(|| InternalError)?;
            if let Some(pte) = src_pt.lookup_page(src_va) {
                let pa = pte.pa();
                let user_attr = Entry::from(ArchPageTableEntry::new(attr)).attribute();
                let attr = user_attr.filter();
                //println!("mem_map [{}][{:016x}] -> [{}][{:016x}] {:?}", src_pid.pid(), src_va, dst_pid.pid(), dst_va, attr);
                let dst_pt = (*dst_pid.pcb()).page_table.ok_or_else(|| InternalError)?;
                dst_pt.insert_page(dst_va, crate::mm::PageFrame::new(pa), attr)?;
                Ok(())
            } else {
                Err(MemoryNotMappedError)
            }
        }
    }

    fn mem_unmap(pid: u16, va: usize) -> Result<(), Error> {
        if va >= CONFIG_USER_LIMIT {
            return Err(MemoryLimitError);
        }
        let p = lookup_pid(pid, true)?;
        unsafe {
            let page_table = (*p.pcb()).page_table.ok_or_else(|| InternalError)?;
            page_table.remove_page(va)?;
        }
        Ok(())
    }

    fn process_alloc() -> Result<Pid, Error> {
        use crate::lib::*;
        unsafe {
            let p = crate::arch::Arch::running_process().unwrap();
            let child = process_pool::alloc(Some(p), 0)?;
            let mut ctx = (*crate::arch::Arch::context()).clone();
            ctx.set_argument(0);
            if cfg!(target_arch = "riscv64") {
                ctx.set_exception_pc(ctx.exception_pc() + 4);
                let pt = (*p.pcb()).page_table.unwrap();
                let c_pt = (*child.pcb()).page_table.unwrap();
                if let Some(pte) = pt.lookup_page(ctx.stack_pointer()) {
                    let frame = crate::mm::page_pool::try_alloc()?;
                    core::intrinsics::volatile_copy_memory(
                        frame.kva() as *mut u8,
                        pa2kva(pte.pa()) as *const u8,
                        PAGE_SIZE,
                    );
                    c_pt.insert_page(ctx.stack_pointer(), frame, EntryAttribute::user_default());
                } else {
                    println!("riscv: stack pointer page fault");
                }
            }

            (*child.pcb()).context = Some(ctx);
            (*child.pcb()).status = crate::lib::process::Status::PsNotRunnable;
            Ok(child.pid())
        }
    }

    fn process_set_status(pid: u16, status: crate::lib::process::Status) -> Result<(), Error> {
        if status != crate::lib::process::Status::PsRunnable
            && status != crate::lib::process::Status::PsNotRunnable
        {
            return Err(InvalidArgumentError);
        }
        let p = lookup_pid(pid, true)?;
        unsafe {
            (*p.pcb()).status = status;
        }
        Ok(())
    }

    fn ipc_receive(dst_va: usize) {
        unsafe {
            let p = crate::arch::Arch::running_process().unwrap();
            (*p.ipc()).attribute = dst_va;
            (*p.ipc()).receiving = true;
            (*p.pcb()).status = crate::lib::process::Status::PsNotRunnable;
            SystemCall::process_yield();
        }
    }

    fn ipc_can_send(pid: u16, value: usize, src_va: usize, attr: usize) -> Result<(), Error> {
        if src_va >= CONFIG_USER_LIMIT {
            return Err(MemoryLimitError);
        }
        let p = lookup_pid(pid, false)?;
        unsafe {
            if !(*p.ipc()).receiving {
                return Err(IpcNotReceivingError);
            }
            if src_va != 0 {
                let dst_va = (*p.ipc()).address;
                if dst_va >= CONFIG_USER_LIMIT {
                    return Err(MemoryLimitError);
                }
                let src_page_table = (*crate::arch::Arch::running_process().unwrap().pcb())
                    .page_table
                    .ok_or_else(|| InternalError)?;
                if let Some(src_pte) = src_page_table.lookup_page(src_va) {
                    let user_attr = Entry::from(ArchPageTableEntry::new(attr)).attribute();
                    let attr = user_attr.filter();
                    (*p.ipc()).attribute = ArchPageTableEntry::from(Entry::new(attr, 0)).to_usize();
                    let dst_page_table = (*p.pcb()).page_table.ok_or_else(|| InternalError)?;
                    dst_page_table.insert_page(dst_va, PageFrame::new(src_pte.pa()), attr)?;
                } else {
                    return Err(InvalidArgumentError);
                }
            }
            (*p.ipc()).receiving = false;
            (*p.ipc()).from = crate::arch::Arch::running_process().unwrap().pid();
            (*p.ipc()).value = value;
            (*p.pcb()).status = crate::lib::process::Status::PsRunnable;
            Ok(())
        }
    }
}
