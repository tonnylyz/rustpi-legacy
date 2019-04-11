.sect .text.start
.global _start
_start:
    mrs x0, mpidr_el1
    and x0, x0, #3
    cbz x0, _start_master
1:  wfe
    b 1b

_start_master:
    ldr x0, =0x1000000
    mov sp, x0
    bl main
    b 1b

.global nop_dummy
nop_dummy:
	nop
	ret
