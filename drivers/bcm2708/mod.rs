/*!
 * This module contains all the implemented BCM 2708 drivers.
 * The covered peripherals are described in
 * https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2835/BCM2835-ARM-Peripherals.pdf
 * They are common to all Raspberry Pi boards.
 */

/// The BCM 2708 peripheral base address.
pub const PERIPHERAL_BASE : usize = 0x3F00_0000;

pub mod gpio;
pub mod uart;
pub mod video_core;
pub mod interrupts;
pub mod system_timer;
pub mod random;
