// To keep this in the first portion of the binary.
.section .text.boot

.globl reset
// Entry point for the kernel as a reset can be thought of as a restart.
reset:
    // Halt all cores except one.
    mrc p15, #0, r4, c0, c0, #5
    and r4, r4, #3
    cmp r4, #0
    bne hang

    // Setup the stack.
    mov sp, #0x8000

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

    // Call memory_init
    bl memory_init

    mov r0, #0x80000000
    add sp, sp, r0
    mcr p15, #0, r0, c12, c0, 0
    add pc, pc, r0
    nop

    // Call kernel_main
    bl kernel_main

.globl hang
// Wait forever
hang:
    wfi
    b hang
