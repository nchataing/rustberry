use core::ptr::{read_volatile, write_volatile};

/// The peripheral base address.
#[cfg(feature = "pi2")] pub const PERIPHERAL_BASE : usize = 0x3F000000;
#[cfg(feature = "pi1")] pub const PERIPHERAL_BASE : usize = 0x20000000;

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
    let mut _c = count;
    unsafe
    {
        asm!
        (
            "1:
                subs $0, $0, #1
                bne 1b"
            : "+r"(_c)
            :
            :
            : "volatile"
        );
    }
}
