SECTIONS
{
    . = 0x00080000;
    .text : {
        *(.text.start)
        *(.text*)
    }
    .data : {
        *(.data*)
    }
    .bss : {
        *(.bss*)
    }
    .rodata : {
        *(.rodata*)
    }
}
