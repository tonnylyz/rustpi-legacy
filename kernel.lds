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
    KERNEL_END = .;
}
