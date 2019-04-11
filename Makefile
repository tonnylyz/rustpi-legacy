.PHONY: all clean cargo

all: cargo kernel8.img

clean:
	cargo clean

cargo:
	cargo xbuild --target aarch64-none-elf.json --release --verbose

kernel8.img: cargo
	aarch64-elf-objcopy target/aarch64-none-elf/release/rustpi -O binary kernel8.img

# My sdcard script
win:
	sudo mount -t drvfs e: /mnt/e
	cp kernel8.img /mnt/e/kernel8.img
	sync
	sleep 1
	sudo umount /mnt/e
	RemoveDrive.exe e: -L