extern "C" {
    #[cfg(target_arch = "riscv64")]
    pub static _binary_user_riscv64_elf_start: [u8; 0x20000];
    #[cfg(target_arch = "aarch64")]
    pub static _binary_user_aarch64_elf_start: [u8; 0x20000];
}
