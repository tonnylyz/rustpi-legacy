CROSS  := aarch64-elf-
CFLAGS := -Wall -ffreestanding
CC     := $(CROSS)gcc
INCLUDES := -I./ -I./include

RUST_CROSS_TARGET   := /home/tonny/rs-build/aarch64-none-elf.json
RUST_CROSS_SYSROOT  := /home/tonny/rs-build/target/sysroot
RUST_CROSS_LIB := $(RUST_CROSS_SYSROOT)/lib/rustlib/aarch64-none-elf/lib/libcore-f30b738416316cf7.rlib \
		    $(RUST_CROSS_SYSROOT)/lib/rustlib/aarch64-none-elf/lib/libcompiler_builtins-64de14a383c0cb63.rlib

OBJECTS :=  start.o main.o

.PHONY: all clean

all: kernel8.img

kernel8.img: $(OBJECTS) $(RUST_CROSS_LIB) kernel.lds
	$(CROSS)ld -o kernel.elf -e _start -T kernel.lds $(OBJECTS) $(RUST_CROSS_LIB)
	$(CROSS)objcopy kernel.elf -O binary kernel8.img

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c $<

%.o: %.rs
	rustc --sysroot $(RUST_CROSS_SYSROOT) --target $(RUST_CROSS_TARGET) -C panic=abort --emit obj $<

clean:
	rm -rf *.o kernel.elf kernel8.img

# My sdcard script
win:
	sudo mount -t drvfs e: /mnt/e
	cp kernel8.img /mnt/e/kernel8.img
	sync
	sleep 1
	sudo umount /mnt/e
	RemoveDrive.exe e: -L