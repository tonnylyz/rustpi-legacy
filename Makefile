PATH := $(HOME)/.cargo/bin:/usr/local/x86_64_aarch64-elf/bin:$(PATH)

.PHONY: all clean kernel

all: kernel kernel8.img

clean:
	cargo clean

kernel:
	cargo xbuild --target aarch64-none-elf.json --release --verbose

kernel8.img: kernel
	aarch64-elf-objcopy target/aarch64-none-elf/release/rustpi -O binary kernel8.img
