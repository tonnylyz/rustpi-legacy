add-symbol-file kernel.elf
target remote | qemu-system-aarch64 -M raspi3 -kernel kernel.elf -display none -S -gdb stdio
set step-mode on
set scheduler-locking step
thread 1
display/i $pc