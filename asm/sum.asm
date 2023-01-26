; Takes the sum of an array

;# requires is_buffer(r1, r2)
;# requires r2 > 1

    mov r3, 0 ; idx
    mov r5, 0

loop:
;# req is_buffer(r1, r2)
;# req r3 < sub(r2, 1)
    mov r4 r1
    add r4 r3
    ldxh r4 [r4]
    add r5 r4

    add r3 2
    mov r6 r2
    sub r6 1
    jlt r3 r6 loop

    mov r0 r5
    exit
