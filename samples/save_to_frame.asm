;# requires is_buffer(r1, r2)
;# requires r2 = 8
;# requires is_buffer(r10, r9)
;# requires r9 = 512
stxdw [r10 - 4] r1
ldxdw r1 [r10 - 4] ; erases knowledge of r1
ldxdw r3 [r1]
exit
