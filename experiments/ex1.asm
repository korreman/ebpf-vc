;# requires is_buffer(r1, r2)
;# requires r2 > 0
;# requires is_buffer(r3, r4)
;# requires r4 > 0

    mov r5 r3
    jle r2 r4 skip
    mov r5 r1
skip:
    add r5 r2
    sub r5 1
    ldxb r0 [r5]
    exit
