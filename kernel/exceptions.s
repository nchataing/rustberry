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
    push    {r0-r3, r12, lr}
    sub     r0, lr, #4
    mrc     p15, 0, r1, c5, c0, 1
    bl      prefetch_abort_handler
    pop     {r0-r3, r12, lr}
    subs    pc, lr, #4

// Data abort
data_abort:
    push    {r0-r3, r12, lr}
    sub     r0, lr, #8
    mrc     p15, #0, r1, c6, c0, #0
    mrc     p15, #0, r2, c5, c0, #0
    bl      data_abort_handler
    pop     {r0-r3, r12, lr}
    subs    pc, lr, #8

// IRQ
irq:
    sub     lr, lr, #4
    srsdb   sp!, #0x13
    cpsid   i, #0x13
    stmdb   sp, {r0-r12, sp, lr}^   // Push all the usr registers
    sub     sp, sp, #60
    mov     r0, sp
    push    {lr}                    // Also push lr_svc

    // Realign stack on 8 byte boundary
    and     r4, sp, #7
    sub     sp, sp, r4

    bl      irq_handler

    add     sp, sp, r4
    pop     {lr}
    ldmia   sp, {r0-r12, sp, lr}^
    add     sp, sp, #60
    rfeia   sp!

// FIQ
fiq:
    sub     lr, lr, #4
    srsdb   sp!, #0x13
    cpsid   if, #0x13
    push    {r0-r3, r12, lr}
    and     r1, sp, #7
    sub     sp, sp, r1
    push    {r1}
    bl      fiq_handler
    pop     {r1}
    add     sp, sp, r1
    pop     {r0-r3, r12, lr}
    rfeia   sp!

