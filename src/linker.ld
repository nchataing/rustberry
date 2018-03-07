MEMORY
{
    start : ORIGIN = 0x0, LENGTH = 0x100
    kernel_prgm : ORIGIN = 0x8000, LENGTH = 0x18000
}

SECTIONS
{
    .text.boot :
    {
        KEEP(*(.text.boot))
    } >start

    .text :
    {
        *(.text*)
    } >kernel_prgm
    . = ALIGN(4096);

    .rodata :
    {
        *(.rodata*)
        *(.data.rel.ro.*)
    } >kernel_prgm
    . = ALIGN(4096);

    .data :
    {
        *(.data*)
    } >kernel_prgm
    . = ALIGN(4096);

    __bss_start = .;
    .bss :
    {
        *(.bss*)
    } >kernel_prgm
    __bss_end = .;
    . = ALIGN(4096);
}
