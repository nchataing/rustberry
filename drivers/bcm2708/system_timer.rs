use crate::bcm2708;
use crate::bcm2708::interrupts;
use crate::mmio;

const TIMER_BASE: usize = (bcm2708::PERIPHERAL_BASE + 0x3000);

const TIMER_STATUS: *mut u32 = (TIMER_BASE + 0x00) as *mut u32;
const TIMER_LOW: *mut u32 = (TIMER_BASE + 0x04) as *mut u32;
const TIMER_HIGH: *mut u32 = (TIMER_BASE + 0x08) as *mut u32;

// Only CMP registers 1 and 3 are available for CPU usage.
const TIMER_CMP1: *mut u32 = (TIMER_BASE + 0x10) as *mut u32;
const TIMER_CMP3: *mut u32 = (TIMER_BASE + 0x18) as *mut u32;

/// Return the system timer in µs trucated at 32 bits.
pub fn get_time_low() -> u32 {
    unsafe { mmio::read(TIMER_LOW) }
}

/// Retern the highest 32 bits of the system timer (in 2^32 µs).
pub fn get_time_high() -> u32 {
    unsafe { mmio::read(TIMER_HIGH) }
}

/// Return the current time in µs by reading the system timer.
pub fn get_time() -> u64 {
    (get_time_high() as u64) << 32 | (get_time_low() as u64)
}

/// There are only two available system timers
pub enum SystemTimer {
    Timer1 = 1,
    Timer3 = 3,
}
pub use self::SystemTimer::*;

/// Set the timer remaining time in µs.
pub fn set_remaining_time(timer: SystemTimer, micro_secs: u32) {
    let current_time = get_time_low();
    let trigger_time = current_time.wrapping_add(micro_secs);
    set_trigger_time(timer, trigger_time);
}

/**
 * Set the timer trigger time in µs.
 * This is the next time at witch the callback is called.
 */
pub fn set_trigger_time(timer: SystemTimer, trigger_time: u32) {
    unsafe {
        match timer {
            Timer1 => mmio::write(TIMER_CMP1, trigger_time),
            Timer3 => mmio::write(TIMER_CMP3, trigger_time),
        }
    }
}

/**
 * Setup the function called when the timer is finished.
 *
 * The callback must call `clear_irq` at the begining.
 * It also should call `set_remaining_time` or `unregister_callback`
 * on its associated timer, to choose between beeing called again later
 * and disabling itself.
 */
pub fn register_callback(id: SystemTimer, callback: fn()) {
    interrupts::register_irq(id as u32, callback);
}

/// Remove a callback for a timer. This effectively disables the timer IRQ.
pub fn unregister_callback(id: SystemTimer) {
    interrupts::unregister_irq(id as u32);
}

/// Clear a timer IRQ. This must be called inside a callback function.
/// See `register_callback`.
pub fn clear_irq(timer: SystemTimer) {
    unsafe {
        mmio::write(TIMER_STATUS, 1 << (timer as u32));
    }
}
