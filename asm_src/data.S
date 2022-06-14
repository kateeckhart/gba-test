.section ".rodata"
.align 4
.global ROOT_DIR
ROOT_DIR:
.incbin "data.bin"
.global ROOT_DIR_SIZE
ROOT_DIR_SIZE:
.word ROOT_DIR_SIZE - ROOT_DIR
