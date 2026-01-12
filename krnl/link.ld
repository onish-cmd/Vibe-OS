/* link.ld */
OUTPUT_ARCH( "riscv" )
ENTRY( _start )

SECTIONS
{
  /* QEMU Virt machine RAM starts here */
  . = 0x80000000;

  /* 1. The Code (including our boot.S _start) */
  .text : {
    KEEP(*(.text._start))
    *(.text .text.*)
  }

  /* 2. Read-only Data (Your "NISH" string) */
  .rodata : {
    *(.rodata .rodata.*)
  }

  /* 3. Initialized Data */
  .data : {
    *(.data .data.*)
  }

  /* 4. Uninitialized Data (BSS) */
  .bss : {
    __bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    __bss_end = .;
  }

  /* 5. The Stack */
  . = ALIGN(16);
  . += 0x4000;      /* 16KB of stack space */
  __stack_top = .;
}