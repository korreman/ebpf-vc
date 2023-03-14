    mov r0 0
    mov r1 0
    mov r2 32
loop:
    add r0 r1
    add r1 1
    jeq r1 r1 loop
    exit
