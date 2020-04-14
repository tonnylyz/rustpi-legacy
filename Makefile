.PHONY: all clean kernel emu

ARM:=1
#RISCV:=1

ifdef ARM
ARCH:= aarch64
CROSS:= ${ARCH}-elf-
endif
ifdef RISCV
ARCH:= riscv64
CROSS:= ${ARCH}-unknown-elf-
endif

kernel:
	cargo build --target target.${ARCH}.json -Zbuild-std=core,alloc --release
	cp target/target.${ARCH}/release/rustpi rustpi.${ARCH}.elf
	${CROSS}objcopy rustpi.${ARCH}.elf -O binary rustpi.${ARCH}.img
	${CROSS}objdump -D rustpi.${ARCH}.elf > debug.${ARCH}.D
	${CROSS}objdump -x rustpi.${ARCH}.elf > debug.${ARCH}.x
	${CROSS}nm -n rustpi.${ARCH}.elf > debug.${ARCH}.nm

emu: kernel
ifdef ARM
	qemu-system-aarch64 -M raspi3 -kernel rustpi.${ARCH}.img -serial null -serial stdio -display none
endif
ifdef RISCV
	qemu-system-riscv64 -M virt -bios default -device loader,file=rustpi.${ARCH}.img,addr=0x80200000 -display none
endif

clean:
	cargo clean
