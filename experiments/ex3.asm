; This should have some simple requirement,
; but contain unconditional jumps back and forth.
; The program is effectively still a straightline program,
; just presented in a non-chronological order.

;# requires is_buffer(r1, r2)
;# requires r2 >= 8

    mov r3 r1
    ja label1
label2:
    sub r3 8
    ja label3
label1:
    sub r3 8
    ja label2
label3:
    ldxdw r3 [r1]
    exit
