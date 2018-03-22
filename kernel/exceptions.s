// Exception vector should be at the begining of the binary.
.section .text.exception_vector

// Interrupt vector table
exception_vector:
    b reset
    b undefined_instruction
    b software_interrupt
    b prefetch_abort
    b data_abort
    nop // Reserved
    b irq
    b fiq

// Here are the low level exception handlers calling Rust handlers
.section .text.exceptions

// Undefined instruction interrupt
undefined_instruction:
    sub     r0, lr, #4
    cps     #0x13
    b       undefined_instruction_handler

// Software interrupt
software_interrupt:
    push    {lr}
    bl      software_interrupt_handler
    pop     {lr}
    movs    pc, lr

// Prefetch abort
prefetch_abort:
    sub     r0, lr, #4
    cps     #0x13
    b       prefetch_abort_handler

// Data abort
data_abort:
    sub     r0, lr, #8
    mrc     p15, #0, r1, c6, c0, #0
    mrc     p15, #0, r2, c5, c0, #0
    cps     #0x13
    b       data_abort_handler

// IRQ
irq:
    sub     lr, lr, #4
    srsdb   sp!, #0x13
    cpsid   i, #0x13
    push    {r0-r3, r12, lr}
    and     r1, sp, #4
    sub     sp, sp, r1
    push    {r1}
    bl      irq_handler
    pop     {r1}
    add     sp, sp, r1
    pop     {r0-r3, r12, lr}
    rfeia   sp!

// FIQ
fiq:
    sub     lr, lr, #4
    srsdb   sp!, #0x13
    cpsid   if, #0x13
    push    {r0-r3, r12, lr}
    and     r1, sp, #4
    sub     sp, sp, r1
    push    {r1}
    bl      fiq_handler
    pop     {r1}
    add     sp, sp, r1
    pop     {r0-r3, r12, lr}
    rfeia   sp!

