;; Simple Euclids GCD, demonstrating control flow

mov r1, 15
mov r2, 27

;; routine
loop:
assert true
mov r3 r1
mod r3 r2
mov r1 r2
mov r2 r3
jne r2 0 loop

;; done
end:
exit
