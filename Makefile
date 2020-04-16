.PHONY: all clean user aarch64 riscv64 aarch64-emu riscv64-emu

AARCH64_CROSS:=aarch64-elf-
RISCV64_CROSS:=riscv64-unknown-elf-

all: aarch64 riscv64

user:
	make -C user

aarch64: user
	cargo build --target target.aarch64.json -Zbuild-std=core,alloc --release
	${AARCH64_CROSS}objcopy target/target.aarch64/release/rustpi -O binary rustpi.aarch64.img

riscv64: user
	cargo build --target target.riscv64.json -Zbuild-std=core,alloc --release
	${RISCV64_CROSS}objcopy target/target.riscv64/release/rustpi -O binary rustpi.riscv64.img

aarch64-emu: aarch64
	qemu-system-aarch64 -M raspi3 -kernel rustpi.aarch64.img -serial null -serial stdio -display none

riscv64-emu: riscv64
	qemu-system-riscv64 -M virt -m 1024 -bios default -device loader,file=rustpi.riscv64.img,addr=0x80200000 -serial stdio -display none

clean:
	cargo clean
