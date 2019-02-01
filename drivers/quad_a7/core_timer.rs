/*!
 * This modules manages the core timers.
 * It assumes that the code is running in Secure PL1 mode.
 *
 * There are two timer for each processor core named physical and virtual.
 * They can be considered as identical unless the CNTVOFF is modified (not
 * possible without going into hyp or mon modes).
 *
 * Timers use the ARM clock as a base but they only advance by one every two
 * instructions (this is called the APB clock).
 *
 * The callbacks are shared by all the cores but each core has its own timer.
 * A core can only set the remaining time or mask its own timers.
 */

use mmio;
use quad_a7;

#[derive(Clone, Copy)]
pub enum CoreTimer {
    Physical = 0,
    Virtual = 3,
}
pub use self::CoreTimer::*;

const CONTROL_REG: *mut u32 = quad_a7::PERIPHERAL_BASE as *mut u32;
const PRESCALER: *mut u32 = (quad_a7::PERIPHERAL_BASE + 0x08) as *mut u32;
const INTERRUPT_BASE: usize = quad_a7::PERIPHERAL_BASE + 0x40;

coproc_reg! {
    CNTP_TVAL : p15, c14, 0, c2, 0;
    CNTP_CTL : p15, c14, 0, c2, 1;
    CNTV_TVAL : p15, c14, 0, c3, 0;
    CNTV_CTL : p15, c14, 0, c3, 1;
}

coproc_reg64! {
    CNTPCT : p15, c14, 0;
    CNTVCT : p15, c14, 1;
}

fn unregistered_timer_handler() {
    panic!("Unregistered core timer event occured")
}

static mut PHYSICAL_TIMER_HANDLER: fn() = unregistered_timer_handler;
static mut VIRTUAL_TIMER_HANDLER: fn() = unregistered_timer_handler;

/**
 * Initialize the core counters.
 * If not called once in one core, the timers will be really slow.
 */
pub fn init() {
    unsafe {
        // Setup core timer to count from APB clock
        mmio::write(CONTROL_REG, 1 << 8);

        // Use directly the clock without prescaling
        mmio::write(PRESCALER, 0x8000_0000);
    }
}

/// Get the current time indicated by the given timer (both should be equal).
pub fn get_time(timer: CoreTimer) -> u64 {
    unsafe {
        match timer {
            Physical => CNTPCT::read(),
            Virtual => CNTVCT::read(),
        }
    }
}

/**
 * Get the remaining time before a timer expires.
 * Each core can have a different remaining time for the same timer.
 */
pub fn get_remaining_time(timer: CoreTimer) -> u32 {
    unsafe {
        match timer {
            Physical => CNTP_TVAL::read(),
            Virtual => CNTV_TVAL::read(),
        }
    }
}

/**
 * Set the remaining time before a timer expires.
 * It is impossible to set the remaining time for another core.
 */
pub fn set_remaining_time(timer: CoreTimer, apb_ticks: u32) {
    unsafe {
        match timer {
            Physical => CNTP_TVAL::write(apb_ticks),
            Virtual => CNTV_TVAL::write(apb_ticks),
        }
    }
}

/**
 * Enables or disables a timer. All timers start disabled.
 * A disabled timer is not stopped but will never be reported as ended
 * (and by consequence will never generate any interrupt).
 * This property can only be changed for the current core.
 */
pub fn set_enabled(timer: CoreTimer, status: bool) {
    unsafe {
        match (timer, status) {
            (Physical, true) => CNTP_CTL::set_bits(1),
            (Physical, false) => CNTP_CTL::reset_bits(1),
            (Virtual, true) => CNTV_CTL::set_bits(1),
            (Virtual, false) => CNTV_CTL::reset_bits(1),
        }
    }
}

/**
 * Mask or unmask a timer. All timers start unmasked.
 * A masked timer can be reported as ended but will never generate any
 * interrupt.
 */
pub fn set_masked(timer: CoreTimer, status: bool) {
    unsafe {
        match (timer, status) {
            (Physical, true) => CNTP_CTL::set_bits(0b10),
            (Physical, false) => CNTP_CTL::reset_bits(0b10),
            (Virtual, true) => CNTV_CTL::set_bits(0b10),
            (Virtual, false) => CNTV_CTL::reset_bits(0b10),
        }
    }
}

/**
 * Check if the specified core-local timer has ended.
 */
pub fn has_ended(timer: CoreTimer) -> bool {
    unsafe {
        match timer {
            Physical => CNTP_CTL::read() & (1 << 3) != 0,
            Virtual => CNTP_CTL::read() & (1 << 3) != 0,
        }
    }
}

/**
 * Register a callback function to trigger when the given timer expires.
 * Note that the same function is used by all cores.
 * If FIQ is set, the generated interruption will be a FIQ instead of an IRQ.
 *
 * If called multiple times, the callback will be replaced.
 */
pub fn register_callback(timer: CoreTimer, handler: fn(), fiq: bool) {
    unsafe {
        match timer {
            Physical => PHYSICAL_TIMER_HANDLER = handler,
            Virtual => VIRTUAL_TIMER_HANDLER = handler,
        }

        for core in 0..4 {
            let reg = (INTERRUPT_BASE + 4 * core) as *mut u32;

            let mut val = mmio::read(reg);
            if fiq {
                val |= 1 << (timer as u32 + 4);
            } else {
                val &= !(1 << (timer as u32 + 4));
                val |= 1 << timer as u32;
            }
            mmio::write(reg, val);
        }
    }
}

/**
 * Unregister a callback function.
 */
pub fn unregister_callback(timer: CoreTimer) {
    unsafe {
        for core in 0..4 {
            let reg = (INTERRUPT_BASE + 4 * core) as *mut u32;

            let mut val = mmio::read(reg);
            val &= !(1 << (timer as u32 + 4) | 1 << timer as u32);
            mmio::write(reg, val);
        }

        match timer {
            Physical => PHYSICAL_TIMER_HANDLER = unregistered_timer_handler,
            Virtual => VIRTUAL_TIMER_HANDLER = unregistered_timer_handler,
        }
    }
}

pub fn handle_interrupt(timer: CoreTimer) {
    unsafe {
        match timer {
            Physical => PHYSICAL_TIMER_HANDLER(),
            Virtual => VIRTUAL_TIMER_HANDLER(),
        }
    }
}
