MEMORY
{
    start : ORIGIN = 0x0, LENGTH = 0x100
    bootloader_prgm : ORIGIN = 0x4000, LENGTH = 0x4000
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
    } >bootloader_prgm

    .rodata :
    {
        *(.rodata*)
        *(.data.rel.ro.*)
    } >bootloader_prgm
}
