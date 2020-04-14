SECTIONS
{
    . = 0xffffffffc0200000;
    .text : {
        *(.text.start)
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
        *(.bss.stack)
        *(.bss.ipc)
        *(.bss*)
    }
    BSS_END = .;
    /DISCARD/ : {
        *(.comment)
    }
    KERNEL_END = .;
}
