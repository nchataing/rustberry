// To keep this in the first portion of the binary.
.section ".text.boot"

// Make start global.
.globl start

// Entry point for the kernel.
start:
    // Halt all cores except one.
    mrc p15, #0, r4, c0, c0, #5
    and r4, r4, #3
    cmp r4, #0
    bne halt

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

    // Enable FPU
    ldr r0, =(0xF << 20)
    mcr p15, 0, r0, c1, c0, 2
    mov r3, #0x40000000
    .long 0xeee83a10 // vmsr FPEXC, r3

    // Call kernel_main
    bl kernel_main

halt:
    wfe
    b halt
