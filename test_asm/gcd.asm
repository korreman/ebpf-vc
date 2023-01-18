;; Simple Euclids GCD, demonstrating control flow
;; r1 - a
;; r2 - b

;; routine
loop:
assert true
jeq r2, 0, end
mov r3 r1
mod r3 r2
mov r1 r2
mov r2 r3
ja loop

;; done
end:
exit
