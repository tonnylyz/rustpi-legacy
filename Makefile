PATH := $(HOME)/.cargo/bin:/usr/local/x86_64_aarch64-elf/bin:$(PATH)

.PHONY: all clean kernel emu miniclean debug

all: kernel

clean:
	cargo clean

kernel:
	cargo xbuild --target aarch64-none-elf.json --release --verbose
	cp target/aarch64-none-elf/release/rustpi kernel.elf
	aarch64-elf-objcopy kernel.elf -O binary kernel8.img

debug:
	aarch64-elf-objdump -D kernel.elf > dis
	aarch64-elf-objdump -x kernel.elf > dissym
	nm -n kernel.elf > dissymlist
	aarch64-elf-gdb -x debug.gdb

emu: kernel
	qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial null -serial stdio -display none