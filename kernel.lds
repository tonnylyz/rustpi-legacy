SECTIONS
{
    . = 0x80000;
    .init : {
        *(.text.start)
        *(.text.kvm)
        *(.data.kvm)
    }
    _init_end = ABSOLUTE(.);
    .text 0xFFFFFF8000000000 + _init_end : AT(_init_end) {
        *(.text*)
        *(.user.text)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
    }
    .bss : {
        *(.bss*)
    }
    /DISCARD/ : {
        *(.comment)
    }
    _kernel_end = .;
}
