.macro VECTOR handler
    csrrw sp, sscratch, sp // save user's sp into sscratch
    addi  sp, sp, -0x120 // size of ContextFrame
    sd x1, 1(sp)
    //sd x2, 2(sp) (this x2 is kernel sp)
    sd x3, 3(sp)
    sd x4, 4(sp)
    sd x5, 5(sp)
    sd x6, 6(sp)
    sd x7, 7(sp)
    sd x8, 8(sp)
    sd x9, 9(sp)
    sd x10, 10(sp)
    sd x11, 11(sp)
    sd x12, 12(sp)
    sd x13, 13(sp)
    sd x14, 14(sp)
    sd x15, 15(sp)
    sd x16, 16(sp)
    sd x17, 17(sp)
    sd x18, 18(sp)
    sd x19, 19(sp)
    sd x20, 20(sp)
    sd x21, 21(sp)
    sd x22, 22(sp)
    sd x23, 23(sp)
    sd x24, 24(sp)
    sd x25, 25(sp)
    sd x26, 26(sp)
    sd x27, 27(sp)
    sd x28, 28(sp)
    sd x29, 29(sp)
    sd x30, 30(sp)
    sd x31, 31(sp)

    csrrw s0, sscratch, zero // user's sp (also clear sscratch)
    csrr s1, sstatus
    csrr s2, sepc
    csrr s3, scause
    csrr s4, stval

    sd s0, 2(sp)
    sd s1, 32(sp)
    sd s2, 33(sp)
    sd s3, 34(sp)
    sd s4, 35(sp)

    mv a0, sp
    jal \handler
    j pop_context

.endm

.global push_context
push_context:
    VECTOR exception_entry // see exception.rs

.global pop_context
pop_context:
    // kernel sp points to bottom of a context frame
    addi s0, sp, 0x120 // size of ContextFrame
    csrw sscratch, s0 // save kernel sp back to sscratch
    ld s1, 32(sp)
    ld s2, 33(sp)
    csrw sstatus, s1
    csrw sepc, s2

    ld x1, 1(sp)
    // no x2(sp) here
    ld x3, 3(sp)
    ld x4, 4(sp)
    ld x5, 5(sp)
    ld x6, 6(sp)
    ld x7, 7(sp)
    ld x8, 8(sp)
    ld x9, 9(sp)
    ld x10, 10(sp)
    ld x11, 11(sp)
    ld x12, 12(sp)
    ld x13, 13(sp)
    ld x14, 14(sp)
    ld x15, 15(sp)
    ld x16, 16(sp)
    ld x17, 17(sp)
    ld x18, 18(sp)
    ld x19, 19(sp)
    ld x20, 20(sp)
    ld x21, 21(sp)
    ld x22, 22(sp)
    ld x23, 23(sp)
    ld x24, 24(sp)
    ld x25, 25(sp)
    ld x26, 26(sp)
    ld x27, 27(sp)
    ld x28, 28(sp)
    ld x29, 29(sp)
    ld x30, 30(sp)
    ld x31, 31(sp)

    ld x2, 2(sp)// restore user sp
    sret

.global get_sp
get_sp:
    mv a0, sp
    ret

.global set_sp
set_sp:
    mv sp, a0
    ret

.global set_sbi_timer
set_sbi_timer:
    ecall
    ret