ENTRY(_start)

SECTIONS
{
    /* Starts at LOADER_ADDR. */
    . = 0x8000;
    .text :
    {
        KEEP(*(.text.boot))
        *(.text)
    }
    . = ALIGN(4096); /* align to page size */

    .rodata :
    {
        *(.rodata)
    }
    . = ALIGN(4096); /* align to page size */

    .data :
    {
        *(.data)
    }
    . = ALIGN(4096); /* align to page size */

    __bss_start = .;
    .bss :
    {
        *(.bss)
    }
    . = ALIGN(4096); /* align to page size */
    __bss_end = .;
}
