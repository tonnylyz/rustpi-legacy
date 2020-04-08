# Bare-metal OS for Raspberry Pi 3 in Rust

Build the kernel:
```
make kernel
```
It will yield a `kernel8.img`, put it on sd card and run it at your Raspberry Pi 3, or run an emulation using QEMU:
```
qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial null -serial stdio -display none
```

More OS design documents are coming!

**User Programs (in Rust)**

* Repo: https://github.com/tonnylyz/rustpi-user
 

**What is working:**
* Bootstrap (written in Rust)
* UART
* Kernel Virtual Memory (code compiled from Rust run at **high** address space, from https://github.com/rust-embedded/rust-raspi3-OS-tutorials/tree/master/11_virtual_memory)
* Kernel interrupt and exception handling (from https://github.com/rust-embedded/rust-raspi3-OS-tutorials/tree/master/12_cpu_exceptions_part1)
* Kernel non-paged pool (buddy system from rCore: https://github.com/rcore-os/buddy_system_allocator)
* User space memory management (paged)
* User programs running at user mode (see https://github.com/tonnylyz/rustpi-user)
* System calls (only putc)
* Memory management system calls (JOS styled)
* Process management system calls (JOS styled)
* A user `fork` demo

**Todo:**
* Code refactoring
* Copy on Write page fault management
* Inter-process communication(IPC) system calls
* and so on...

**Building & emulating on Windows!**
1. Linaro GCC Toolchain: https://releases.linaro.org/components/toolchain/binaries/latest-7/aarch64-elf/gcc-linaro-7.5.0-2019.12-i686-mingw32_aarch64-elf.tar.xz
2. GnuWin32 coreutils & make: http://gnuwin32.sourceforge.net/packages.html
3. QEMU: https://qemu.weilnetz.de/w64/
4. Rust Nightly Toolchain: https://rustup.rs/

