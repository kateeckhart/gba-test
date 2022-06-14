.arm
.section ".fast_text", "x"
.global __aeabi_memcpy
.type __aeabi_memcpy, %function
__aeabi_memcpy:
.global __aeabi_memcpy4
.type __aeabi_memcpy4, %function
__aeabi_memcpy4:
.global __aeabi_memcpy8
.type __aeabi_memcpy8, %function
__aeabi_memcpy8:
.global memcpy
.type memcpy, %function
memcpy:
ORR R3, R0, R1
ORR R3, R3, R2
TST R3, #3
BNE memcpy_unalligned
STMFD R13!, {R0,R5-R11} // R0 store_loc, R3 end_loc, R1 source_loc, R2 count
BIC R3, R2, #31
ADD R3, R0, R3
B memcpy_fast_loop_end
memcpy_fast_loop:
LDMIA R1!, {R5-R12}
STMIA R0!, {R5-R12}
memcpy_fast_loop_end:
CMP R3, R0
BNE memcpy_fast_loop
AND R2, R2, #31 // R2 bytes remaining
MOV R2, R2, LSL #1 // R2 bytes remaining * 2
RSB R2, R2, #52 // R2 jump offset 
ADD R15, R2
.rept 7
LDR R5, [R1], #4
STR R5, [R3], #4
.endr
LDMFD R13!, {R0,R5-R11}
BX R14
memcpy_unalligned:
MOVS R2, R2
BXEQ R14
MOV R3, R0
memcpy_unalligned_loop:
LDRB R12, [R1], #1
STRB R12, [R0], #1
SUBS R2, R2, #1
BNE memcpy_unalligned_loop
MOV R0, R3
BX R14

