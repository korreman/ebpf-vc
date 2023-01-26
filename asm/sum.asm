; Takes the sum of an array

;# requires r2 > 0
;# requires is_buffer(r1, r2)

    mov r3, 0
    mov r5, 0
loop:
;# req is_buffer(r1, r2)
;# req r3 < r2
    mov r4 r1
    add r4 r3
    ldxdw r4 [r4]
    add r5 r4

    add r3 1
    jne r3 r2 loop

    mov r0 r5
    exit
