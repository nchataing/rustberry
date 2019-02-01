/*!
 * This modules defines the Quad-A7 mailbox interface for inter-core
 * communication.
 * There are 4 different mailboxes for each cores.
 *
 * A mailbox emits an interruption as long as it is non null.
 *
 * We have choosen here to stick to a fully symetric model for mailbox handlers:
 * all cores share the same handlers.
 * This implies that the same mailbox have the same role on each core.
 */

use mmio;
use quad_a7;

const MAILBOX_INTERRUPT_CONTROL_BASE: usize = quad_a7::PERIPHERAL_BASE + 0x50;
const MAILBOX_SET_BASE: usize = quad_a7::PERIPHERAL_BASE + 0x80;
const MAILBOX_READ_BASE: usize = quad_a7::PERIPHERAL_BASE + 0xC0;

fn disabled_interrupt_handler() {
    panic!("Unregistered mailbox interrupt occured")
}

static mut MAILBOX_INTERRUPT_HANDLERS: [fn(); 4] = [disabled_interrupt_handler; 4];

/**
 * Read the mailbox `mailbox_id` of the core `core_id`.
 * There is no problem reading the mailbox of another core.
 */
pub fn read(core_id: u8, mailbox_id: u8) -> u32 {
    assert!(core_id < 4);
    assert!(mailbox_id < 4);

    let reg = MAILBOX_READ_BASE + (core_id * 0x10 + mailbox_id * 0x4) as usize;
    unsafe { mmio::read(reg as *const u32) }
}

/**
 * Write bits to the mailbox `mailbox_id` of the core `core_id`.
 * This do not clear any bit already high.
 * It means that it behaves like a `mailbox |= value`.
 */
pub fn write(core_id: u8, mailbox_id: u8, value: u32) {
    assert!(core_id < 4);
    assert!(mailbox_id < 4);

    let reg = MAILBOX_SET_BASE + (core_id * 0x10 + mailbox_id * 0x4) as usize;
    unsafe {
        mmio::write(reg as *mut u32, value);
    }
}

/**
 * Clear bits of the mailbox `mailbox_id` of the core `core_id`.
 * It behaves like a `mailbox &= !value`.
 */
pub fn clear(core_id: u8, mailbox_id: u8, value: u32) {
    assert!(core_id < 4);
    assert!(mailbox_id < 4);

    let reg = MAILBOX_READ_BASE + (core_id * 0x10 + mailbox_id * 0x4) as usize;
    unsafe {
        mmio::write(reg as *mut u32, value);
    }
}

/**
 * Register a mailbox handler callback that will be called when the mailbox
 * is not filled with zeroes.
 * Note that the handler must clear the mailbox before returning,
 * otherwise it will be triggered continuously.
 * If there is already an handler for the same mailbox, it will be replaced.
 * Note that you can choose if the interruption generated is an IRQ or a FIQ.
 */
pub fn register_callback(mailbox_id: u8, handler: fn(), fiq: bool) {
    unsafe {
        MAILBOX_INTERRUPT_HANDLERS[mailbox_id as usize] = handler;
        for core in 0..4 {
            let reg = (MAILBOX_INTERRUPT_CONTROL_BASE + core * 4) as *mut u32;
            let mut val = mmio::read(reg);
            if fiq {
                val |= 1 << (mailbox_id + 4);
            } else {
                val &= !(1 << (mailbox_id + 4));
                val |= 1 << mailbox_id;
            }
            mmio::write(reg, val);
        }
    }
}

/// Disable interruptions for the specified mailbox.
pub fn unregister_callback(mailbox_id: u8) {
    unsafe {
        for core in 0..4 {
            let reg = (MAILBOX_INTERRUPT_CONTROL_BASE + core * 4) as *mut u32;
            let mut val = mmio::read(reg);
            val &= !(1 << (mailbox_id + 4) | 1 << mailbox_id);
            mmio::write(reg, val);
        }
        MAILBOX_INTERRUPT_HANDLERS[mailbox_id as usize] = disabled_interrupt_handler;
    }
}

pub fn handle_interrupt(mailbox_id: u8) {
    unsafe { MAILBOX_INTERRUPT_HANDLERS[mailbox_id as usize]() }
}
