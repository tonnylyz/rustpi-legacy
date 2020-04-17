use riscv::regs::*;

use crate::arch::{PAGE_SHIFT, PAGE_SIZE};
use crate::lib::page_table::PageTableTrait;
use crate::lib::process::Process;
use crate::lib::scheduler::{RoundRobinScheduler, SchedulerTrait};
use crate::mm::PageFrame;

pub type Arch = Riscv64Arch;

pub type ContextFrame = super::context_frame::Riscv64ContextFrame;

pub type PageTable = super::page_table::Riscv64PageTable;

pub type ArchPageTableEntry = super::page_table::Riscv64PageTableEntry;

pub type AddressSpaceId = u16;

pub struct Riscv64Arch;

pub static mut CONTEXT: Option<usize> = None;
static mut PROCESS: Option<Process> = None;
static mut SCHEDULER: RoundRobinScheduler = RoundRobinScheduler::new();

impl crate::arch::ArchTrait for Riscv64Arch {
    fn exception_init() {
        super::exception::init();
    }

    fn start_first_process() -> ! {
        use core::intrinsics::size_of;
        extern "C" {
            fn pop_context_first(ctx: usize) -> !;
        }
        unsafe {
            let context = (*crate::arch::Arch::running_process().unwrap().pcb())
                .context
                .unwrap();
            pop_context_first(&context as *const ContextFrame as usize)
        }
    }

    fn kernel_page_table() -> PageTable {
        let ppn = SATP.read(SATP::PPN) as usize;
        PageTable::new(PageFrame::new(ppn << PAGE_SHIFT))
    }

    fn user_page_table() -> PageTable {
        // Note user and kernel share page directory
        Self::kernel_page_table()
    }

    fn set_user_page_table(pt: PageTable, asid: AddressSpaceId) {
        unsafe {
            SATP.write(
                SATP::MODE::Sv39
                    + SATP::ASID.val(asid as u64)
                    + SATP::PPN.val((pt.directory().pa() >> PAGE_SHIFT) as u64),
            );
            riscv::barrier::sfence_vma_all();
        }
    }

    fn invalidate_tlb() {
        riscv::barrier::sfence_vma_all();
    }

    fn wait_for_event() {
        riscv::asm::wfi();
    }

    fn nop() {
        riscv::asm::nop();
    }

    fn fault_address() -> usize {
        STVAL.get() as usize
    }

    fn core_id() -> usize {
        0
    }

    fn context() -> *mut ContextFrame {
        unsafe { CONTEXT.unwrap() as *mut ContextFrame }
    }

    fn has_context() -> bool {
        unsafe { CONTEXT.is_some() }
    }

    fn running_process() -> Option<Process> {
        unsafe { PROCESS }
    }

    fn set_running_process(p: Option<Process>) {
        unsafe {
            PROCESS = p;
        }
    }

    fn schedule() {
        unsafe {
            SCHEDULER.schedule();
        }
    }
}
