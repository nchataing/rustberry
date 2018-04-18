use core::ptr::{read_volatile, write_volatile};

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
#[inline] pub fn mem_barrier()
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
#[inline] pub fn sync_barrier()
{
    unsafe
    {
        asm!("dsb" :::: "volatile")
    }
}

/**
 * Instruction Synchronization Barrier
 * Flushes the pipeline in the processor, so that all instructions following
 * the ISB are fetched from cache or memory.
 */
#[inline] pub fn instr_barrier()
{
    unsafe
    {
        asm!("isb" :::: "volatile")
    }
}

/// Waits until set_event is called from another core or an interruption occur.
#[inline] pub fn wait_for_event()
{
    unsafe
    {
        asm!("wfe" :::: "volatile")
    }
}

/// Wake up other processors if they are inside wait_for_event
#[inline] pub fn set_event()
{
    unsafe
    {
        asm!("sev" :::: "volatile")
    }
}


