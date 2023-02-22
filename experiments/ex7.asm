; Bubble sort
; Demonstrates a nested loop with memory access

; r3: index counter
; r4: epoch counter
; r5: address temporary
; r6: preceding element temporary
; r7: current element temporary
; r8: size of valid addresses

;# requires is_buffer(r1, r2)
;# requires r2 >= 16

    mov r4 0
    mov r8 r2
    sub r8 7
outer_loop:
;# req r8 = sub(r2, 7)
;# req is_buffer(r1, r2)
;# req r2 >= 16
;# req r4 < r2
    mov r3 8

inner_loop:
;# req r8 = sub(r2, 7)
;# req is_buffer(r1, r2)
;# req 8 <= r3
;# req r3 < r8
    mov r5 r3
    add r5 r1
    ldxdw r6 [r5 - 8]
    ldxdw r7 [r5]
    jlt r6 r7 skipped
    stxdw [r5 - 8] r7
    stxdw [r5] r6
skipped:
    add r3 8
    jlt r3 r8 inner_loop
    add r4 8
    jlt r4 r2 outer_loop

    mov r0 0
    exit
