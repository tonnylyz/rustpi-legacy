.macro VECTOR handler
    sub	sp, sp, #0x110 // size of ContextFrame
    stp x0, x1,  [sp, #(0 * 16)]
    stp x2, x3,  [sp, #(1 * 16)]
    stp x4, x5,  [sp, #(2 * 16)]
    stp x6, x7,  [sp, #(3 * 16)]
    stp x8, x9,  [sp, #(4 * 16)]
    stp x10,x11, [sp, #(5 * 16)]
    stp x12,x13, [sp, #(6 * 16)]
    stp x14,x15, [sp, #(7 * 16)]
    stp x16,x17, [sp, #(8 * 16)]
    stp x18,x19, [sp, #(9 * 16)]
    stp x20,x21, [sp, #(10 * 16)]
    stp x22,x23, [sp, #(11 * 16)]
    stp x24,x25, [sp, #(12 * 16)]
    stp x26,x27, [sp, #(13 * 16)]
    stp x28,x29, [sp, #(14 * 16)]
    mrs x1, spsr_el1
    stp x30, x1, [sp, #(15 * 16)]
    mrs x0, elr_el1
    mrs x1, sp_el0
    stp x0, x1,  [sp, #(16 * 16)]
    mov x0, sp
    bl  \handler
    b   pop_context
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

.global pop_context_first
pop_context_first:
    mov sp, x0
.global pop_context
pop_context:
    ldr x0, [sp, #(31 * 8)] // spsr
    ldr x1, [sp, #(32 * 8)] // elr
    ldr x2, [sp, #(33 * 8)] // sp
    msr spsr_el1, x0
    msr elr_el1, x1
    msr sp_el0, x2
    ldp x0, x1,  [sp, #(0 * 16)]
    ldp x2, x3,  [sp, #(1 * 16)]
    ldp x4, x5,  [sp, #(2 * 16)]
    ldp x6, x7,  [sp, #(3 * 16)]
    ldp x8, x9,  [sp, #(4 * 16)]
    ldp x10,x11, [sp, #(5 * 16)]
    ldp x12,x13, [sp, #(6 * 16)]
    ldp x14,x15, [sp, #(7 * 16)]
    ldp x16,x17, [sp, #(8 * 16)]
    ldp x18,x19, [sp, #(9 * 16)]
    ldp x20,x21, [sp, #(10 * 16)]
    ldp x22,x23, [sp, #(11 * 16)]
    ldp x24,x25, [sp, #(12 * 16)]
    ldp x26,x27, [sp, #(13 * 16)]
    ldp x28,x29, [sp, #(14 * 16)]
    ldr x30, [sp, #(15 * 16)]
    add	sp, sp, #0x110 // size of ContextFrame
    eret

.section .text.kvm
.global smpen
smpen:
    mrs x0, S3_1_c15_c2_1
    orr x0, x0, #0x40
    msr S3_1_c15_c2_1, x0
    ret