SECTIONS
{
    . = 0xFFFFFFFF80200000;
    .text : {
        *(.text.start)
        *(.data.start)
        *(.text*)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
    }
    . = ALIGN(4096);
    BSS_START = .;
    .bss : {
        *(.bss.ipc)
        *(.bss*)
    }
    BSS_END = .;
    /DISCARD/ : {
        *(.comment)
    }
    KERNEL_END = .;
}
