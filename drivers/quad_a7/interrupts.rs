use crate::bcm2708;
use crate::mmio;
use crate::quad_a7;

const IRQ_SOURCE_BASE: usize = quad_a7::PERIPHERAL_BASE + 0x60;
const FIQ_SOURCE_BASE: usize = quad_a7::PERIPHERAL_BASE + 0x70;

const GPU_INTERRUPT_ROUTING: *mut u32 = (quad_a7::PERIPHERAL_BASE + 0x0C) as *mut u32;

/**
 * Initialize the interrupt system.
 * This must be called only once by only one core.
 * It also enable IRQ and FIQ.
 */
pub fn init() {
    bcm2708::interrupts::init();
    enable_all();
}

pub fn enable_irq() {
    unsafe { asm!("cpsie i" :::: "volatile") }
}

pub fn disable_irq() {
    unsafe { asm!("cpsid i" :::: "volatile") }
}

pub fn enable_fiq() {
    unsafe { asm!("cpsie f" :::: "volatile") }
}

pub fn disable_fiq() {
    unsafe { asm!("cpsid f" :::: "volatile") }
}

/// Enable IRQ and FIQ
pub fn enable_all() {
    unsafe { asm!("cpsie if" :::: "volatile") }
}

/// Disable IRQ and FIQ
pub fn disable_all() {
    unsafe { asm!("cpsid if" :::: "volatile") }
}

pub fn handle_irq() {
    let proc_id = quad_a7::get_core_id() as usize;
    let irq_source_reg = (IRQ_SOURCE_BASE + 4 * proc_id) as *const u32;
    let irq_source = unsafe { mmio::read(irq_source_reg) };

    if irq_source & (1 << 8) != 0 {
        bcm2708::interrupts::handle_irq();
    }
    handle_interrupt(irq_source);
}

pub fn handle_fiq() {
    let proc_id = quad_a7::get_core_id() as usize;
    let fiq_source_reg = (FIQ_SOURCE_BASE + 4 * proc_id) as *const u32;
    let fiq_source = unsafe { mmio::read(fiq_source_reg) };

    if fiq_source & (1 << 8) != 0 {
        bcm2708::interrupts::handle_fiq();
    }
    handle_interrupt(fiq_source);
}

// This function handle IRQ/FIQ agnostic interrupt dispatch
fn handle_interrupt(int_source: u32) {
    if int_source & (1 << 0) != 0 {
        quad_a7::core_timer::handle_interrupt(quad_a7::core_timer::Physical);
    }
    if int_source & (1 << 3) != 0 {
        quad_a7::core_timer::handle_interrupt(quad_a7::core_timer::Virtual);
    }

    for mailbox_id in 0..4 {
        if int_source & (1 << (mailbox_id + 4)) != 0 {
            quad_a7::mailbox::handle_interrupt(mailbox_id);
        }
    }
}

/**
 * Define which core should recieve the GPU interrupts.
 * By default it is core 0 for both IRQ and FIQ.
 */
pub fn set_gpu_interrupt_routing(irq_core_id: u8, fiq_core_id: u8) {
    assert!(irq_core_id < 4);
    assert!(fiq_core_id < 4);

    let val = (irq_core_id | (fiq_core_id << 2)) as u32;
    unsafe {
        mmio::write(GPU_INTERRUPT_ROUTING, val);
    }
}
