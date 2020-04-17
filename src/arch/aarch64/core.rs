use crate::board::BOARD_CORE_NUMBER;
use crate::lib::process::Process;
use crate::lib::scheduler::RoundRobinScheduler;

#[derive(Copy, Clone)]
pub struct Core {
    pub context: usize,
    pub running_process: Option<Process>,
    pub scheduler: RoundRobinScheduler,
}

pub static mut CORES: [Core; BOARD_CORE_NUMBER] = [Core {
    context: 0,
    running_process: None,
    scheduler: RoundRobinScheduler::new(),
}; BOARD_CORE_NUMBER];
