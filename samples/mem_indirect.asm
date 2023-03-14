;# requires is_buffer(r1, r2)
;# requires r2 = 32
    mov r0 0
    jge r4 1000 skip   ; avoid overflows
    mov r6 r4
    mul r6 2
    jge r4 32 skip     ; r2 < 32 => r6 < 64
    mov r5 r1
    add r5 r6
    ldxb r0 [r5]
skip:
    exit
