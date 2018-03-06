MEMORY
{
    start : ORIGIN = 0x0, LENGTH = 0x100
    kernel : ORIGIN = 0x8000, LENGTH = 0x18000
}

SECTIONS
{
    .text.boot :
    {
        KEEP(*(.text.boot))
    } >start

    /* Starts at LOADER_ADDR. */
    .text BLOCK(4096) :
    {
        *(.text)
    } >kernel

    .rodata BLOCK(4096) :
    {
        *(.rodata)
    } >kernel

    .data BLOCK(4096) :
    {
        *(.data)
    } >kernel

    __bss_start = .;
    .bss BLOCK(4096) :
    {
        *(.bss)
    } >kernel
    __bss_end = .;
}
