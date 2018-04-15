MEMORY
{
    start : ORIGIN = 0x80000000, LENGTH = 0x100
    /* Kernel should occupy a section (check mem/mmu.rs if size increased) */
    kernel_prgm : ORIGIN = 0x80008000, LENGTH = 0xF8000
}

SECTIONS
{
    .text.start :
    {
        start = .;
        KEEP(*(.text.exception_vector))
        *(.text.boot)
    } >start

    .text :
    {
        __text = .;
        *(.text*)
    } >kernel_prgm

    .rodata ALIGN(4096) :
    {
        __rodata = .;
        *(.rodata*)
        *(.data.rel.ro.*)
    } >kernel_prgm

    .data ALIGN(4096) :
    {
        __data = .;
        *(.data*)
    } >kernel_prgm

    .bss ALIGN(4096) :
    {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    } > kernel_prgm

    . = ALIGN(4096);
    __end = .;
}
