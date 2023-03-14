    jgt r2 1000 end
    mov r3 0
loop:
    add r3 1
    jlt r3 r2 loop
end:
    exit
