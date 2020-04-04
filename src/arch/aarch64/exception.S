#.equ KERNEL_STACK_TOP, 0xFFFFFF8000080000

.macro VECTOR handler
    str x30, [sp, #-16]!
    ldr x30, =CONTEXT_FRAME
    stp x0, x1,  [x30], #16
    stp x2, x3,  [x30], #16
    stp x4, x5,  [x30], #16
    stp x6, x7,  [x30], #16
    stp x8, x9,  [x30], #16
    stp x10,x11, [x30], #16
    stp x12,x13, [x30], #16
    stp x14,x15, [x30], #16
    stp x16,x17, [x30], #16
    stp x18,x19, [x30], #16
    stp x20,x21, [x30], #16
    stp x22,x23, [x30], #16
    stp x24,x25, [x30], #16
    stp x26,x27, [x30], #16
    stp x28,x29, [x30], #16

    ldr x0, [sp], #16 // x30
    mrs x1, spsr_el1
    stp x0, x1, [x30], #16

    mrs x0, elr_el1
    mrs x1, sp_el0
    stp x0, x1, [x30], #16

    #ldr x0, =KERNEL_STACK_TOP
    #mov sp, x0
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

.global pop_time_stack
pop_time_stack:
    ldr x30, =CONTEXT_FRAME
    ldr x0, [x30, #(31 * 8)] // spsr
    ldr x1, [x30, #(32 * 8)] // elr
    ldr x2, [x30, #(33 * 8)] // sp
    msr spsr_el1, x0
    msr elr_el1, x1
    msr sp_el0, x2
    ldp x0, x1,  [x30], #16
    ldp x2, x3,  [x30], #16
    ldp x4, x5,  [x30], #16
    ldp x6, x7,  [x30], #16
    ldp x8, x9,  [x30], #16
    ldp x10,x11, [x30], #16
    ldp x12,x13, [x30], #16
    ldp x14,x15, [x30], #16
    ldp x16,x17, [x30], #16
    ldp x18,x19, [x30], #16
    ldp x20,x21, [x30], #16
    ldp x22,x23, [x30], #16
    ldp x24,x25, [x30], #16
    ldp x26,x27, [x30], #16
    ldp x28,x29, [x30], #16
    ldr x30, [x30]
    eret