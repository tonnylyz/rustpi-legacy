.section .user.text
.align 12
.global user_program_entry
.org 4096
user_program_entry:
1:  add x0, x0, #1
    svc #0
    //str x0, [x0] // cause an data abort
    b   1b

.end