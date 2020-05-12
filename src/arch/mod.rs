#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;
#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;
pub use self::traits::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "riscv64")]
mod riscv64;
mod traits;
pub mod common;
