SECTIONS {
  . = 0x08000000;
  header . : ALIGN(4) { KEEP(*(.header)) }
  read_only . : ALIGN(4) { *(.text) *(.text.*) *(.rodata) *(.rodata.*) }
  exidx . : ALIGN(4) { *(.ARM.exidx.text.*) }

  . = 0x03000000;
  fast_mem . : ALIGN(4)  { KEEP(*(.fast_text)) }

  __fast_mem_size = SIZEOF(fast_mem);
  __fast_mem_begin = ADDR(fast_mem);

  . = 0x2000000;
  modify . : ALIGN(4) { KEEP(*(.force_modify)) *(.data) *(.data.*) }
  bss . : ALIGN(4) { *(.bss) *(.bss.*) }

  __malloc_begin = ALIGN(8);

  __modify_size = SIZEOF(modify);
  __modify_begin = ADDR(modify);
}
