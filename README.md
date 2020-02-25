# Bare-metal OS for Raspberry Pi 3 in Rust

**What is working:**
* Bootstrap (written in assembly and C)
* UART
* Kernel Virtual Memory (code compiled from Rust run at **high** address space, from https://github.com/rust-embedded/rust-raspi3-OS-tutorials/tree/master/11_virtual_memory)
* Kernel interrupt and exception handling (from https://github.com/rust-embedded/rust-raspi3-OS-tutorials/tree/master/12_cpu_exceptions_part1)

**Todo:**
* User space memory management (paged)
* User programs running at user mode
* System calls
* and so on...


