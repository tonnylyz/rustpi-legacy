.equ KERNEL_STACK_TOP, 0xFFFFFF8000080000
.equ TRAP_FRAME_SIZE, 280

.macro VECTOR handler
    str x0, [sp, #-16]!
    ldr x0, =TRAP_FRAME
    add x0, x0, TRAP_FRAME_SIZE
    stp x29, x30, [x0, #-16]!
    stp x27, x28, [x0, #-16]!
    stp x25, x26, [x0, #-16]!
    stp x23, x24, [x0, #-16]!
    stp x21, x22, [x0, #-16]!
    stp x19, x20, [x0, #-16]!
    stp x17, x18, [x0, #-16]!
    stp x15, x16, [x0, #-16]!
    stp x13, x14, [x0, #-16]!
    stp x11, x12, [x0, #-16]!
    stp x9, x10, [x0, #-16]!
    stp x7, x8, [x0, #-16]!
    stp x5, x6, [x0, #-16]!
    stp x3, x4, [x0, #-16]!
    stp x1, x2, [x0, #-16]!
    mov x21, x0
    ldr x0, [sp], #16
    mrs x20, sp_el0
    stp x20, x0, [x21, #-16]!
    mrs x19, esr_el1
    mrs x20, elr_el1
    stp x20, x19, [x21, #-16]!
    mrs x19, spsr_el1
    str x19, [x21, #-8]!

    ldr x19, =KERNEL_STACK_TOP
    mov sp, x19
    bl  \handler
    b   pop_time_stack
.endm

.macro VECTOR_DISABLED
1:  wfe
    b      1b
.endm

.align 11
.global vectors
vectors:

// Current exception level with SP_EL0.
.org 0x000
    VECTOR current_el0_synchronous
.org 0x080
    VECTOR current_el0_irq
.org 0x100
    VECTOR_DISABLED
.org 0x180
    VECTOR current_el0_serror

// Current exception level with SP_ELx, x > 0.
.org 0x200
    VECTOR current_elx_synchronous
.org 0x280
    VECTOR current_elx_irq
.org 0x300
    VECTOR_DISABLED // FIQ
.org 0x380
    VECTOR current_elx_serror

// Lower exception level, aarch64
.org 0x400
    VECTOR lower_aarch64_synchronous
.org 0x480
    VECTOR lower_aarch64_irq
.org 0x500
    VECTOR_DISABLED // FIQ
.org 0x580
    VECTOR lower_aarch64_serror

// Lower exception level, aarch32
.org 0x600
    VECTOR_DISABLED
.org 0x680
    VECTOR_DISABLED
.org 0x700
    VECTOR_DISABLED
.org 0x780
    VECTOR_DISABLED
.org 0x800

pop_time_stack:
    ldr x30, =TRAP_FRAME
    ldr x0, [x30], #8
    msr spsr_el1, x0
    ldp x0, x1, [x30], #16
    msr elr_el1, x0
    ldp x1, x0, [x30], #16
    msr sp_el0, x1
    ldp x1, x2, [x30], #16
    ldp x3, x4, [x30], #16
    ldp x5, x6, [x30], #16
    ldp x7, x8, [x30], #16
    ldp x9, x10, [x30], #16
    ldp x11, x12, [x30], #16
    ldp x13, x14, [x30], #16
    ldp x15, x16, [x30], #16
    ldp x17, x18, [x30], #16
    ldp x19, x20, [x30], #16
    ldp x21, x22, [x30], #16
    ldp x23, x24, [x30], #16
    ldp x25, x26, [x30], #16
    ldp x27, x28, [x30], #16
    ldp x29, x30, [x30] // do NOT add `#0` here which makes it an invalid instruction
    eret