.arm
.section ".text.atomic_swap_byte", "ax", %progbits
.type atomic_swap_byte, %function
.align 4
.global atomic_swap_byte
atomic_swap_byte:
SWPB R2, R1, [R0]
MOV R0, R2
BX R14

.thumb_func
.section ".text.__aeabi_unwind_cpp_pr0", "ax", %progbits
.type __aeabi_unwind_cpp_pr0, %function
.global __aeabi_unwind_cpp_pr0
__aeabi_unwind_cpp_pr0:
B __aeabi_unwind_cpp_pr0

.thumb_func
.section ".text.__sync_synchronize", "ax", %progbits
.type __sync_synchronize, %function
.global __sync_synchronize
__sync_synchronize:
BX R14

