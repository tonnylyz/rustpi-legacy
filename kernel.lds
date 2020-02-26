SECTIONS
{
    . = 0x80000;
    .init : {
        *(.text.start)
        *(.text.vm)
        *(.data.vm)
    }
    _init_end = ABSOLUTE(.);
    .text 0xFFFFFF8000000000 + _init_end : AT(_init_end) {
        *(.text*)
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
    _kernel_end = ABSOLUTE(.);
    /DISCARD/ : {
        *(.comment)
    }
}
