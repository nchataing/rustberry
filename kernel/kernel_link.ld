MEMORY
{
    start : ORIGIN = 0x0, LENGTH = 0x100
    kernel_prgm : ORIGIN = 0x8000, LENGTH = 0x1F8000
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
        *(.text*)
    } >kernel_prgm

    .rodata ALIGN(4096) :
    {
        *(.rodata*)
        *(.data.rel.ro.*)
    } >kernel_prgm

    .data ALIGN(4096) :
    {
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
