/*!
 * This is the driver for the VC/ARM mailbox drivers.
 * It is intended for communications with the VC GPU.
 */

use crate::bcm2708;
use crate::mmio;

/// Mailbox 0 base address (Read by ARM)
const MAIL0_BASE: usize = bcm2708::PERIPHERAL_BASE + 0xB880;

const MAIL0_READ: *mut u32 = (MAIL0_BASE + 0x00) as *mut u32;
const MAIL0_PEAK: *mut u32 = (MAIL0_BASE + 0x10) as *mut u32;
const MAIL0_SENDER: *mut u32 = (MAIL0_BASE + 0x14) as *mut u32;
const MAIL0_STATUS: *mut u32 = (MAIL0_BASE + 0x18) as *mut u32;
const MAIL0_CONFIG: *mut u32 = (MAIL0_BASE + 0x1C) as *mut u32;

/// Mailbox 1 base address (Read by GPU)
const MAIL1_BASE: usize = MAIL0_BASE + 0x20;

const MAIL1_WRITE: *mut u32 = (MAIL1_BASE + 0x00) as *mut u32;
const MAIL1_PEAK: *mut u32 = (MAIL1_BASE + 0x10) as *mut u32;
const MAIL1_SENDER: *mut u32 = (MAIL1_BASE + 0x14) as *mut u32;
const MAIL1_STATUS: *mut u32 = (MAIL1_BASE + 0x18) as *mut u32;
const MAIL1_CONFIG: *mut u32 = (MAIL1_BASE + 0x1C) as *mut u32;

// MAILBOX_FULL is the 31th bit of MAILBOX_STATUS
const MAILBOX_FULL: u32 = 1 << 31;
// MAILBOX_EMPTY is the 30th bit of MAILBOX_STATUS
const MAILBOX_EMPTY: u32 = 1 << 30;

pub fn receive(channel: u8) -> Option<u32> {
    unsafe {
        let mut count: usize = 0;

        loop {
            mmio::mem_barrier();
            while mmio::read(MAIL0_STATUS) & MAILBOX_EMPTY != 0 {
                // The program won't wait forever.
                count += 1;
                if count > 0x10_0000 {
                    return None;
                }
            }

            mmio::mem_barrier();
            let data = mmio::read(MAIL0_READ);
            mmio::mem_barrier();

            // Check it is the correct channel
            if (data & 0b1111) as u8 == channel {
                return Some(data & !0b1111);
            }
        }
    }
}

pub fn send<T>(channel: u8, data: *mut T) {
    unsafe {
        mmio::mem_barrier();
        while mmio::read(MAIL1_STATUS) & MAILBOX_FULL != 0 {}
        mmio::mem_barrier();
        mmio::write(MAIL1_WRITE, (data as u32) | (channel as u32));
    }
}
