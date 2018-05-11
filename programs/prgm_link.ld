ENTRY(start)

SECTIONS
{
    . = 0x80000000;

    .text ALIGN(4096) :
    {
        *(.text*)
    }

    .rodata ALIGN(4096) :
    {
        *(.rodata*)
        *(.data.rel.ro.*)
    }

    .data ALIGN(4096) :
    {
        *(.data*)
    }

    .bss ALIGN(4096) :
    {
        *(.bss*)
        *(COMMON)
    }
}
