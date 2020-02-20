.sect .text.start
.global _start

_start:
    mrs x0, mpidr_el1
    and x0, x0, #3
    cbz x0, 2f
1:  wfe
    b 1b

2:  ldr x1, =0x80000
    mov sp, x1
    ldr x1, =vm_init
    blr x1
    mrs x0, currentel
    cmp x0, #4 // EL1?
    beq _el1_start

    // EL2 TO EL1
    ldr x0, =0xFFFFFF8000080000
    msr sp_el1, x0

    mov x0, #0x33ff
    msr cptr_el2, x0
    msr hstr_el2, xzr
    mov x0, #3 << 20
    msr cpacr_el1, x0

    mov x0, #(1 << 31)
    msr hcr_el2, x0
    mov x0, #0x0800
    movk x0, #0x30d0, lsl #16
    msr sctlr_el1, x0

    mov x0, #0x5
    msr spsr_el2, x0
    adr x0, _el1_start
    msr elr_el2, x0
    eret

_el1_start:
    // load exception vector
    //ldr x0, =vectors
    //msr vbar_el1, x0

    ldr x0, =0x440488
    msr mair_el1, x0

    ldr x0, =0x2BA593A19
    msr tcr_el1, x0
    isb

    ldr x0, =0x00200000
    msr ttbr0_el1, x0
    msr ttbr1_el1, x0

    dsb ish
    isb
    mrs x0, sctlr_el1
    ldr x0, =0x30d51825
    msr sctlr_el1, x0
    isb

    ldr x0, =freemem
    ldr x0, [x0]
    ldr x1, =main
    blr x1
1:  wfe
    b 1b

