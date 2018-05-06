/*!
 * This module is a driver for the hardware random generator on the Raspberry
 * Pi 2.
 *
 * This is inspired from Linux code at drivers/char/hw_random/bcm2835-rng.c
 */

use mmio;
use bcm2708;

const RNG_BASE : usize = bcm2708::PERIPHERAL_BASE + 0x104000;

const RNG_CTRL : *mut u32 = (RNG_BASE + 0x00) as *mut u32;
const RNG_STATUS : *mut u32 = (RNG_BASE + 0x04) as *mut u32;
const RNG_DATA : *mut u32 = (RNG_BASE + 0x08) as *mut u32;
const RNG_INT_MASK : *mut u32 = (RNG_BASE + 0x10) as *mut u32;

// the initial numbers generated are "less random" so will be discarded
const RNG_WARMUP_COUNT : u32 = 0x4_0000;

/// Initialize the hardware random engine
pub fn init()
{
    unsafe
    {
        mmio::write(RNG_STATUS, RNG_WARMUP_COUNT);
        mmio::write(RNG_CTRL, 1);
    }
}

/// Generate a random word
pub fn generate() -> Option<u32>
{
    unsafe
    {
        // mmio::read(RNG_STATUS) >> 24 = nb_available_words
        if timeout_wait_while!((mmio::read(RNG_STATUS) >> 24) == 0, 0x1000)
        {
            None
        }
        else
        {
            Some(mmio::read(RNG_DATA))
        }
    }
}
