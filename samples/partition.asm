; The following algorithm sorts a boolean array (zero and non-zero)
; by the ordering 'False<True'.

;# requires is_buffer(r1, r2)
;# requires r2 > 0
; let ref i = 0 in
    mov r3 0
; let ref j = length a - 1 in
    mov r4 r2
    sub r4 1
; while i < j do
loop:
;# req is_buffer(r1, r2)
;# req r4 < r2
;( req 0 <= r3 is needed for variant)
; invariant { j < length a}
; invariant { 0 <= i }
; variant { j - i }
    jle r4 r3 return
; if not a[i] then incr i
case_a:
    mov r5 r1
    add r5 r3
    ldxb r5 [r5]
    jeq r5 0 case_b
    add r3 1
    ja continue
; else if a[j] then decr j
case_b:
    mov r6 r1
    add r6 r4
    ldxb r6 [r6]
    jne r6 0 case_c
    sub r4 1
    ja continue
; else begin
;   swap a i j;
;   incr i;
;   decr j
case_c:
    mov r7 r1
    add r7 r3
    stxb [r7] r6
    mov r7 r1
    add r7 r4
    stxb [r7] r5
; end
continue:
    ja loop
return:
    exit
; done
