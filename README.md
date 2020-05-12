# Rustpi: a research kernel

Build the kernel:
```
make # both aarch64 and riscv64
make aarch64 # for raspberry pi 3b
make riscv64 # for riscv64 qemu machine virt
```
Run emulation:
```
make aarch64-emu # raspberry pi 3b
make riscv64-emu # riscv64 qemu machine virt
```

**User mode programs**
* User mode programs are written in Rust.
* See `build.rs` for how to embed them into kernel image. 
* Repo: https://github.com/tonnylyz/rustpi-user


**What is working (both aarch64 and riscv64)**
* Bootstrap
* UART
* Kernel virtual memory (code compiled from Rust run at **high** address space)
* Kernel interrupt and exception handling
* Kernel non-paged pool (buddy system from rCore: https://github.com/rcore-os/buddy_system_allocator)
* User space memory management (paged)
* User programs running at user mode
* System calls
* Memory management system calls
* Process management system calls
* A user `fork` demo
* Copy on Write page fault management

**Todo**
* A robust scheduler
* Inter-process communication(IPC) for new thread/process model
* Ram disk
* File system
* Code comments
* Code refactoring
* and so on...

**Toolchains required**
* Make
* Rust (latest nightly)
* Aarch64 GCC Toolchain (default prefix: `aarch64-elf-`)
* Riscv64 GCC Toolchain (default prefix: `riscv64-unknown-elf-`)
* QEMU system emulation (`qemu-system-aarch64` and `qemu-system-riscv64`)

Note for Windows: GnuWin32 `coreutils` & `make`: http://gnuwin32.sourceforge.net/packages.html

**Toolchains suggested**
* Aarch64: Linaro GCC Toolchain: https://www.linaro.org/downloads/
* Riscv64: SiFive GCC Toolchain: https://www.sifive.com/boards/
* QEMU: https://www.qemu.org/download/
* Rust: https://rustup.rs/

Note for Windows: toolchains above are also available for Windows.
