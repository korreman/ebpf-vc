;; Solution to day 1, part 1 of Advent of Code 2022.
;; The requires an
;; r0 - return value, max elf
;; r1 - input ptr
;; r2 - input size
;; r3 - index
;; r4 - load dst
;; r5 - number parsing accumulator
;; r6 - current elf

;# requires is_buffer(r1, r2)

mov r0 0
mov r3 0
mov r5 0
mov r6 0

outer: ;; loop
;# req is_buffer(r1, r2)
    jge r3 r2 submit
    mov r4 r1 ;; load next byte
    add r4 r3
    ldxb r4 [r4]
    add r3 1
    jeq r4 10 submit ;; newline check

inner: ;; loop. parses a number from a decimal string, terminated by newline
;# req is_buffer(r1, r2)
    mul r5 10
    add r5 r4
    sub r5 48
    jge r3 r2 submit
    mov r4 r1 ;; load next byte
    add r4 r3
    ldxb r4 [r4]
    add r3 1
    jne r4 10 inner ;; newline check

    add r6 r5
    mov r5 0
    ja outer

submit: ;; finishes an elf. compare to current max and replace if better, reset elf to 0
    jgt r0 r6 skip
    mov r0 r6
skip:
    mov r6 0
    jlt r3 r2 outer
    exit
