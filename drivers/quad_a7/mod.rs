/*!
 * This module contains drivers for the Quad-A7 implementation on a BCM 2836
 * chip.
 * It contains code for communication between processor cores and interruptions.
 */

pub const PERIPHERAL_BASE : usize = 0x4000_0000;

coproc_reg!
{
    MPIDR : p15, c0, 0, c0, 5;
}

pub fn get_core_id() -> u8
{
    unsafe
    {
        MPIDR::read() as u8
    }
}

pub mod interrupts;
pub mod mailbox;
