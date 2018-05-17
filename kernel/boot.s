// To keep this in the first portion of the binary.
.section .text.boot

.globl reset
// Entry point for the kernel as a reset can be thought of as a restart.
reset:
    // Halt all cores except one.
    mrc p15, #0, r4, c0, c0, #5
    and r4, r4, #3
    cmp r4, #0
    bne idle

    // Clear out bss.
    ldr r4, =__bss_start
    ldr r9, =__bss_end
    mov r5, #0
    mov r6, #0
    mov r7, #0
    mov r8, #0
    b       2f

1:
    // store multiple at r4.
    stmia r4!, {r5-r8}

    // If we are still below bss_end, loop.
2:
    cmp r4, r9
    blo 1b

    // Setup the memory map
    mov sp, #0x8000
    bl init_memory_map

    // Set abort mode stack
    cps #0x17
    mov sp, #0x2000

    // Set supervisor mode stack
    cps #0x13
    mov sp, #0x80000000

    // Call kernel_main
    bl kernel_main

.globl idle
// Wait forever
idle:
    wfi
    b idle
