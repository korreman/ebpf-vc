; This should exercise memory access without branching

;# requires is_buffer(r1, r2)
;# requires r2 >= 64

ldxdw r3 [r1]
ldxw r4 [r1 + 8]
ldxh r5 [r1 + 12]
ldxb r6 [r1 + 14]

stxdw [r1 + 16] r3
stxw [r1 + 24] r4
stxh [r1 + 28] r5
stxb [r1 + 30] r6

stdw [r1 + 32] 1
stw [r1 + 40] 2
sth [r1 + 44] 3
stb [r1 + 46] 4

exit
