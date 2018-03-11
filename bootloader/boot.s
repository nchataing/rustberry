// To keep this in the first portion of the binary.
.section ".text.boot"

// Make start global.
.globl start

// Entry point for the bootloader
start:
    // Separate core 0 from other.
    mrc p15, #0, r4, c0, c0, #5
    and r4, r4, #3
    cmp r4, #0
    bne other_core

    // Setup the stack.
    mov sp, #0x4000

    // Call bootloader_main
    bl bootloader_main

.section ".text"

.globl reset

other_core:
    // Wait for signal at 0x2000 before reset
    mov r1, #0x2000
    mov r0, #0
    str r0, [r1]
1:
    ldr r0, [r1]
    cmp r0, #0
    beq 1b

reset:
    // Invalidate data cache
    mov r0, #0
    mcr p15, 0, r0, c7, c10, 1
    dsb

    // Invalidate instruction cache
    mcr p15, 0, r0, c7, c5, 0
    mcr p15, 0, r0, c7, c5, 6
    dsb
    isb

    // Jump to 0x0
    mov r0, #0
    bx r0

