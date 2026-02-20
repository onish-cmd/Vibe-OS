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
    /* MOVE THIS TO THE TOP */
    . = 0xffffffff80000000;

    /* Now all sections start at the high address */
    .requests : {
        KEEP(*(.requests_start_marker))
        KEEP(*(.requests))
        KEEP(*(.requests_end_marker))
    } :data

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

    /* Discard unnecessary stuff that bloats the binary */
    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
