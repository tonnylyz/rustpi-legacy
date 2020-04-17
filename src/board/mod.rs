#[cfg(target_arch = "x86_64")]
pub use self::riscv::*;

#[cfg(target_arch = "aarch64")]
pub use self::rpi3::*;

#[cfg(target_arch = "riscv64")]
pub use self::riscv::*;

pub use self::traits::*;

mod rpi3;
mod riscv;
mod traits;
