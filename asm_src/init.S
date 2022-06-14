.arm
.section ".header", "x"
.global __header
__header:
B __start // Entry
.incbin "nintendo_logo.bin" // Nintendo Logo
.asciz "KATE'S GBA" // Game Title
.byte 0
.ascii "XKAT" // Game Code
.ascii "KT" // Maker Code
.byte 0x96 // Fixed value
.byte 0 // Main unit
.byte 0 // Device type
.skip 7 // Reserved
.byte 0 // Version
.byte 0 // Checksum
.skip 2 // Reserved

.global __start
__start:
LDR R14, thumb_start_loc
BX R14
thumb_start_loc:
.word thumb_start + 1

.text
.thumb
thumb_start:
LDR R7, stack_end_loc
MOV R13, R7
LDR R0, load_fast_mem_begin
LDR R1, fast_mem_begin
LDR R2, fast_mem_size
fast_mem_copy:
LDR R3, [R0]
STR R3, [R1]
ADDS R0, #4
ADDS R1, #4
SUBS R2, R2, #4
BNE fast_mem_copy
LDR R0, modify_begin
LDR R1, load_modify_begin
LDR R2, modify_size
BL memcpy
BL main

.align 4
.global __load_fast_mem_begin
__load_fast_mem_begin:
load_fast_mem_begin:
.word 0
fast_mem_begin:
.word __fast_mem_begin
fast_mem_size:
.word __fast_mem_size
modify_begin:
.word __modify_begin
.global __load_modify_begin
__load_modify_begin:
load_modify_begin:
.word 0
modify_size:
.word __modify_size

stack_end_loc:
.word stack_end

.section ".force_modify"
.asciz "Test str"

.bss
.align 8
stack_start:
.skip 16 * 1024
stack_end:
