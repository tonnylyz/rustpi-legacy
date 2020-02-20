SECTIONS
{
    . = 0x80000;
    _init : {
        *(.text.start)
        *(.text.vm_init)
        *(.data.vm_init)
    }
    . = 0xFFFFFF8000080400;
    .text : AT(0x80400) {
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
    /DISCARD/ : {
        *(.comment)
    }
}
