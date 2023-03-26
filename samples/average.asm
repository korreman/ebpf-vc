;# requires and(is_buffer(r1, r2), r2 >= 4)
; initialization
    mov r3 r1
    add r3 r2
    sub r3 4       ; r3 = r1 + r2 - 4;
    mov r4 r1      ; r4 = r1;
    mov r0 0       ; r0 = 0;
loop:
;# req and(is_buffer(r1, r2), r2 >= 4)
;# req r3 = sub(add(r1, r2), 4)
;# req and(r1 <= r4, r4 <= r3)
    ldxw r5 [r4]   ; r5 = *r4;
    add r0 r5      ; r0 += r5;
    add r4 4       ; r4 += 4;
    jle r4 r3 loop ; if r4 <= r3 goto loop;
; end
    div r0 r2      ; r0 /= r2
    exit
