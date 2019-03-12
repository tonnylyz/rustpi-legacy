CROSS  := aarch64-elf-
CFLAGS := -Wall -ffreestanding
CC     := $(CROSS)gcc
INCLUDES := -I./ -I./include

rust_target   := /usr/local/rs-build/aarch64-none-elf.json
rust_sysroot  := /usr/local/rs-build/target/sysroot
rust_lib := $(rust_sysroot)/lib/rustlib/aarch64-none-elf/lib/libcore-8bb115cd02decc25.rlib \
		    $(rust_sysroot)/lib/rustlib/aarch64-none-elf/lib/libcompiler_builtins-15aebfb075ac2436.rlib

objects :=  start.o main.o

.PHONY: all clean

all: kernel8.img

kernel8.img: $(objects) $(rust_lib) kernel.lds
	$(CROSS)ld -o kernel.elf -e _start -T kernel.lds $(objects) $(rust_lib)
	$(CROSS)objcopy kernel.elf -O binary kernel8.img


%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c $<

%.o: %.rs
	rustc --sysroot $(rust_sysroot) --target $(rust_target) -C panic=abort --emit obj $<

clean:
	rm -rf *.o kernel.elf kernel8.img
