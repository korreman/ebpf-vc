    mov r1 r3
    jeq r2, 0, end
loop:
    ;# req r2 <> 0
    mov r3 r1
    mod r3 r2
    mov r1 r2
    mov r2 r3
    jne r2, 0, loop

end:
    mov r1 r0
    exit
