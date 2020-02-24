PATH := $(HOME)/.cargo/bin:/usr/local/x86_64_aarch64-elf/bin:$(PATH)

.PHONY: all clean kernel emu miniclean

all: kernel

miniclean:
	rm -rf target/release
	rm -rf target/aarch64-none-elf/release

clean:
	cargo clean

kernel: miniclean
	cargo xbuild --target aarch64-none-elf.json --release --verbose
	cp target/aarch64-none-elf/release/rustpi kernel.elf
	aarch64-elf-objcopy kernel.elf -O binary kernel8.img

emu: kernel
	qemu-system-aarch64 -M raspi3 -kernel kernel8.img -serial null -serial stdio -display none -gdb tcp::1234