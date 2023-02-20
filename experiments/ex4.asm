; This program exhibits branching logic without having any cycles.
; Demonstrates that (in-program) annotations aren't needed
; unless the program contains cycles.

;# requires is_buffer(r1, r2)
;# requires r2 > 0
;# requires is_buffer(r3, r4)
;# requires r4 > 0

    mov r5 r3
    jle r2 r4 skipped
    mov r5 r1
skipped:
    add r5 r2
    sub r5 1
    ldxb r0 [r5]
    exit
