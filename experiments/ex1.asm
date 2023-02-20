; This one should exercise module-wide pre- and post-conditions, arithmetic,
; and div/modulo safety.

;# requires 0 <= r1
;# requires r1 < r2

mov r4 r2
sub r4 r1
add r3 493
mul r3 504
mod r3 r4
mov r0 r3
exit
