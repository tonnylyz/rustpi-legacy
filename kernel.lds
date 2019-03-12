SECTIONS
{
    . = 0x00080000;
    .text : {
        start.o
        *(.text*)
    }
    .bss : {
        *(.bss*)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
    }
}
