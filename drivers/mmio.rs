use core::ptr::{read_volatile, write_volatile};

/// The peripheral base address.
#[cfg(feature = "pi2")] pub const PERIPHERAL_BASE : usize = 0x3F000000;
#[cfg(feature = "pi1")] pub const PERIPHERAL_BASE : usize = 0x20000000;

/// Memory mapped read
#[inline] pub unsafe fn read(reg: *const u32) -> u32
{
    read_volatile(reg)
}

/// Memory mapped write
#[inline] pub unsafe fn write(reg: *mut u32, data: u32)
{
    write_volatile(reg, data)
}

/// Loop <count> times in a way that the compiler won't optimize away
#[inline] pub fn delay(count: u32)
{
    let mut _c = count;
    unsafe
    {
        asm!
        (
            "1:
                subs $0, $0, #1
                bne 1b"
            : "+r"(_c)  ::: "volatile"
        );
    }
}

/**
 * Data memory barrier
 * No memory access after the DMB can run until all memory accesses before it
 * have completed
 */
pub fn mem_barrier()
{
    unsafe
    {
        asm!("dmb" :::: "volatile")
    }
}

/**
 * Data synchronisation barrier
 * No instruction after the DSB can run until all instructions before it have
 * completed
 */
pub fn sync_barrier()
{
    unsafe
    {
        asm!("dsb" :::: "volatile")
    }
}

/**
 * Clean and invalidate entire cache
 * Flush pending writes to main memory
 * Remove all data in data cache
 */
pub fn flush_cache()
{
    unsafe
    {
        asm!(
            "mcr p15, #0, $0, c7, c14, #1
             mcr p15, #0, $0, c7, c14, #2"
             :: "r"(0) :: "volatile")
    }
}
