ENTRY(_start)

PHDRS
{
    text    PT_LOAD FLAGS(5); /* 5 = Read + Execute */
    rodata  PT_LOAD FLAGS(4); /* 4 = Read Only */
    data    PT_LOAD FLAGS(6); /* 6 = Read + Write */
    dynamic PT_DYNAMIC FLAGS(6);
}

SECTIONS
{
  .requests : {
        KEEP(*(.requests_start_marker))
        KEEP(*(.requests))
        KEEP(*(.requests_end_marker))
    } :data
    /* Load address for the higher half */
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(0x1000);

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(0x1000);

    .data : {
        *(.data .data.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    .dynamic : {
        KEEP(*(.dynamic))
    } :data :dynamic
}
