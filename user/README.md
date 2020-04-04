#User space program objects

Generate with:
```
aarch64-elf-ld -r -b binary -o user.o user.elf
```
Copy `user.o` to this folder.

It creates symbol like:
```
0000000000000000 D _binary_user_elf_start
0000000000010400 D _binary_user_elf_end
0000000000010400 A _binary_user_elf_size
``` 