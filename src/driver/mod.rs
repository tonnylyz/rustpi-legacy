pub mod mmio;

#[cfg(target_arch = "x86_64")]
mod riscv;
#[cfg(target_arch = "x86_64")]
pub use self::riscv::*;

#[cfg(target_arch = "aarch64")]
mod rpi3;
#[cfg(target_arch = "aarch64")]
pub use self::rpi3::*;

#[cfg(target_arch = "riscv64")]
mod riscv;
#[cfg(target_arch = "riscv64")]
pub use self::riscv::*;
