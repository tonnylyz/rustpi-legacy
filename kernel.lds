SECTIONS
{
    . = 0x00080000;
    .text : {
        start.o
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
