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
    . = 0xffffffff80000000;

    /* 1. CODE SECTION */
    .text : {
        *(.text .text.*)
    } :text

    /* FORCE ALIGNMENT TO 4KB BEFORE SWITCHING PHDR */
    . = ALIGN(0x1000); 

    /* 2. READ-ONLY DATA SECTION */
    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    /* FORCE ALIGNMENT AGAIN */
    . = ALIGN(0x1000); 

    /* 3. WRITABLE DATA SECTION */
    .data : {
        *(.data .data.*)
        *(.requests_start_marker)
        *(.requests)
        *(.requests_end_marker)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /* 4. DYNAMIC SECTION (If needed, also aligned) */
    . = ALIGN(0x1000);
    .dynamic : {
        KEEP(*(.dynamic))
    } :data :dynamic
}
