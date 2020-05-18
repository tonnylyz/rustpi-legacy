.section .text.start
.global _start
_start:
    lui   sp, %hi(BOOT_STACK_TOP)
    addi  sp, sp, %lo(BOOT_STACK_TOP)
    addiw sp, sp, 0 // sp sign-extended to 64 bit

    mv    t0, a0 // t0 <- hartid
    slli  t0, t0, 17 // t0 <- hartid * 0x20000
    subw  sp, sp, t0
    csrrw x0, sscratch, a0 // sscratch <- hartid

    lui   t0, %hi(KERNEL_PAGE_DIRECTORY)
    addi  t0, t0, %lo(KERNEL_PAGE_DIRECTORY)
    slli  t0, t0, 32
    srli  t0, t0, 32 // truncate t0 to 32 bits
    srli  t0, t0, 12 // pa -> ppn
    # Note: Supervisor Adress Translation and Protection (satp) Register
    #  [   8   ][   16    ][      44      ]  ~ 64 bit
    #    mode      asid          PPN(physical page number)
    # we choose `mode` == 8
    # which means page-based 39-bit virtual addressing (Sv39)
    li    t1, 8
    slli  t1, t1, 60
    or    t0, t0, t1 // mod = 8
    csrw  satp, t0
    sfence.vma
    lui   t0, %hi(hart_spin)
    addi  t0, t0, %lo(hart_spin)
    addiw t0, t0, 0
    jr    t0

.section .data.start
.align 12
.global KERNEL_PAGE_DIRECTORY
KERNEL_PAGE_DIRECTORY:
    # 0x0000_0000_8000_0000 -> 0x8000_0000 (1G)
    # 0xffff_ffff_0000_0000 -> 0x0000_0000 (1G)
    # 0xffff_ffff_4000_0000 -> 0x4000_0000 (1G)
    # 0xffff_ffff_8000_0000 -> 0x8000_0000 (1G)
    # 0xffff_ffff_c000_0000 -> 0xc000_0000 (1G)
    # Note Sv39 page table entry:
    # [   10   ][  44 ][ 2 ][1][1][1][1][1][1][1][1]
    #  Reserved   PPN   RSW  D  A  G  U  X  W  R  V
    # *Important*: PPN place at bit 10 not bit 12
    # pte = ((pa >> 12) << 10) | other bits
    # V: valid
    # XWR:
    #    000 pointer to next level of page table
    #    001 read-only page
    #    010 reserved
    #    011 read-write page
    #    100 execute-only page
    #    101 read-execute page
    #    110 reserved
    #    111 read-write-execute page
    # U: user mode accessible
    # G: global mapping (exist in all address spaces)
    # A: accessed (leaf PTE) clear to trigger page fault
    # D: dirty (leaf PTE) clear to trigger page fault
    # RSW: supervisor defined
    # Note: also see sstatus::SUM bit (supervisor mode can also access pages with U=1)
    .quad 0
    .quad 0
    .quad ((0x80000000 >> 12) << 10) | 0xcf # VRWXAD
    .zero 8 * 505
    .quad ((0x00000000 >> 12) << 10) | 0xcf # VRWXAD
    .quad ((0x40000000 >> 12) << 10) | 0xcf # VRWXAD
    .quad ((0x80000000 >> 12) << 10) | 0xcf # VRWXAD
    .quad ((0xc0000000 >> 12) << 10) | 0xcf # VRWXAD

.section .data.start
.align 12
BOOT_STACK:
    .space 4096 * 32 * 4
.global BOOT_STACK_TOP
BOOT_STACK_TOP:
