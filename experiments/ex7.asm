; Bubble sort

;# requires is_buffer(r1, mul(r2, 8))
;# requires r2 > 1

; r3: index counter
; r4: epoch counter
; r5: address
; r6: preceding element location
; r7: current element location

    mov r4 0
outer_loop:
;# req is_buffer(r1, mul(r2, 8))
;# req r4 < r2
    mov r3 1

inner_loop:
;# req is_buffer(r1, mul(r2, 8))
;# req r3 < r2
    mov r5 r3
    mul r5 8
    add r5 r1
; load the current and preceding value
    ldxdw r6 [r5 - 8]
    ldxdw r7 [r5]
; swap them if they are ordered incorrectly
    jlt r6 r7 skipped
    stxdw [r5 - 8] r7
    stxdw [r5] r6
skipped:
    add r3 1
    jlt r3 r2 inner_loop
    add r4 1
    jlt r4 r2 outer_loop

    mov r0 0
    exit
