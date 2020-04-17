SECTIONS
{
    . = 0x80000;
    .init : {
        *(.text.start)
        *(.text.kvm)
        *(.data.kvm)
    }
    INIT_END = ABSOLUTE(.);
    .text 0xFFFFFF8000000000 + INIT_END : AT(INIT_END) {
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
