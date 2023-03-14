;# ensures r0 >= r2
;# ensures r0 >= r3
;# ensures r0 >= r4
a:
    jlt r2 r3 c
b:
    jge r2 r4 R2
c:
    jge r3 r4 R3
    ja R4
R2:
    mov r0 r2
    jmp end
R3:
    mov r0 r3
    jmp end
R4:
    mov r0 r4
end:
    exit
