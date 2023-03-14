;# requires is_buffer(r1, r2)
;# requires r2 > 1
    mov r3, 0 ; idx
    mov r0, 0
loop:
;# req is_buffer(r1, r2)
;# req r3 < sub(r2, 1)
    mov r4 r1
    add r4 r3
    ldxh r4 [r4]
    add r0 r4      ; load element and add to sum
    add r3 2
    mov r6 r2
    sub r6 1
    jlt r3 r6 loop ; loop if address is 2 lower than size
    mov r0 r0
    exit
