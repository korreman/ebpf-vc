;# requires is_buffer(r1, r2)
    jlt r3 4 end
    jge r3 32 end
    jle r3 12 load
    jlt r3 20 end  ; if r3 in [4;12] U [20;32)
load:              ; how do i make it work for me?
    ldxdw r0 [r3]
end:
    exit
