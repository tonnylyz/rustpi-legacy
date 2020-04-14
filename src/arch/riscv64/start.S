.section .text.start
.global _start
_start:
    lui  sp, %hi(boot_stack_top)
    addi sp, sp, %lo(boot_stack_top)
    # satp = (8 << 60) | PPN(boot_page_table_sv39)
    lui  t0, %hi(boot_page_table_sv39)
    li   t1, 0xffffffffc0000000 - 0x80000000
    sub  t0, t0, t1
    srli t0, t0, 12
    li   t1, 8 << 60
    or   t0, t0, t1
    csrw satp, t0
    sfence.vma

    lui  t0, %hi(main)
    addi t0, t0, %lo(main)
    jr   t0

.section .bss.stack
.align 12
boot_stack:
    .space 4096 * 16
.global boot_stack_top
boot_stack_top:

.section .data
.align 12
boot_page_table_sv39:
    # 0x00000000_80000000 -> 0x80000000 (1G)
    # 0xffffffff_c0000000 -> 0x80000000 (1G)
    .quad 0
    .quad 0
    .quad (0x80000 << 10) | 0xcf # VRWXAD
    .zero 8 * 508
    .quad (0x80000 << 10) | 0xcf # VRWXAD