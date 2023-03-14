;# requires is_buffer(r1, r2)
;# requires r2 > 0
    mov r3 0           ; i = 0
    mov r4 r2
    sub r4 1           ; j = len array - 1
loop:
;# req is_buffer(r1, r2)
;# req r4 < r2
;( req 0 <= r3 is needed for termination)
    jle r4 r3 return   ; while i < j
caseA:
    mov r5 r1
    add r5 r3
    ldxb r5 [r5]
    jne r5 0 caseB     ; if a[i] <> 0 goto B;
    add r3 1           ; i += 1; continue;
    ja continue
caseB:
    mov r6 r1
    add r6 r4
    ldxb r6 [r6]
    jeq r6 0 caseC     ; if a[j] == 0 goto C;
    sub r4 1           ; j -= 1; continue;
    ja continue
caseC:
    mov r7 r1
    add r7 r3
    stxb [r7] r6
    mov r7 r1
    add r7 r4
    stxb [r7] r5       ; a[i] <-> a[j]
; end
continue:
    ja loop
return:
    exit
; done
