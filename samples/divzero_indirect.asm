    mov r0 0
    jeq r2 r3 skip  ; Ensure that r2 <> r3
    mov r1 r2
    sub r1 r3
    mov r0 r4
    div r0 r1       ; We indirectly know that r2 - r3 <> 0
skip:
    exit
