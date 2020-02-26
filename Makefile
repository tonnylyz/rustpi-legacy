#PATH := $(HOME)/.cargo/bin:/usr/local/x86_64_aarch64-elf/bin:$(PATH)

.PHONY: all clean kernel emu debug

all: kernel

clean:
	cargo clean

kernel:
	cargo xbuild --target aarch64-none-elf.json --release --verbose
	cp target/aarch64-none-elf/release/rustpi kernel.elf
	aarch64-elf-objcopy kernel.elf -O binary kernel8.img

debug: kernel
	aarch64-elf-objdump -D kernel.elf > debug.D
	aarch64-elf-objdump -x kernel.elf > debug.x
	aarch64-elf-nm -n kernel.elf > debug.nm
	aarch64-elf-gdb -x debug.gdb

emu: kernel
	qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial null -serial stdio -display none