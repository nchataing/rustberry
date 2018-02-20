use core::ptr::{read_volatile, write_volatile};

/// The peripheral base address.
#[cfg(feature = "raspi2")] pub const PERIPHERAL_BASE : usize = 0x3F000000;
#[cfg(feature = "raspi1")] pub const PERIPHERAL_BASE : usize = 0x20000000;

/// The GPIO registers base address.
pub const GPIO_BASE : usize = (PERIPHERAL_BASE + 0x200000);

/// Controls actuation of pull up/down to ALL GPIO pins.
pub const GPPUD : *mut i32  = (GPIO_BASE + 0x94) as *mut i32;

/// Controls actuation of pull up/down for specific GPIO pin.
pub const GPPUDCLK0 : *mut i32 = (GPIO_BASE + 0x98) as *mut i32;

/// Memory mapped read
#[inline] pub unsafe fn read(reg: *const i32) -> i32
{
    read_volatile(reg)
}

/// Memory mapped write
#[inline] pub unsafe fn write(reg: *mut i32, data: i32)
{
    write_volatile(reg, data)
}

/// Loop <count> times in a way that the compiler won't optimize away
#[inline] pub fn delay(count: i32)
{
    unsafe
    {
        asm!
        (
            "1:
                subs $0, $0, #1
                bne 1b"
            :
            : "r"(count)
            :
            : "volatile"
        );
    }
}
