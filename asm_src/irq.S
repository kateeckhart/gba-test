.arm
.section ".fast_text"
.type irq_handle, %function
.align 4
.global irq_handle
irq_handle:
LDR R0, irq_rec_loc
LDR R1, irq_ack_const
STRH R1, [R0]
LDR R0, timer_loc
LDR R1, [R0]
ADDS R1, R1, #1
STR R1, [R0]
BX R14

irq_ack_const:
.word 0x40
irq_rec_loc:
.word 0x4000202
timer_loc:
.word TIMER_VALUE
