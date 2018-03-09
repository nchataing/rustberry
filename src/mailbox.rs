use mmio;

pub const MAIL0_BASE : usize = 0x3F00B880;

pub const MAIL0_READ   : *mut u32 = (MAIL0_BASE + 0x00) as *mut u32;
pub const MAIL0_PEAK   : *mut u32 = (MAIL0_BASE + 0x10) as *mut u32;
pub const MAIL0_SENDER : *mut u32 = (MAIL0_BASE + 0x14) as *mut u32;
pub const MAIL0_STATUS : *mut u32 = (MAIL0_BASE + 0x18) as *mut u32;
pub const MAIL0_CONFIG : *mut u32 = (MAIL0_BASE + 0x1C) as *mut u32;
pub const MAIL0_WRITE  : *mut u32 = (MAIL0_BASE + 0x20) as *mut u32;

// MAILBOX_FULL is the 31th bit of MAILBOX_STATUS
pub const MAILBOX_FULL  : u32 = 1<<31;
// MAILBOX_EMPTY is the 30th bit of MAILBOX_STATUS
pub const MAILBOX_EMPTY : u32 = 1<<30;

pub fn receive(channel: u8) -> Option<u32>
{
    unsafe
    {
        let mut count : usize = 0;
        let mut not_empty = true;

        loop
        {
            while not_empty
            {
                mmio::mem_barrier();
                if mmio::read(MAIL0_STATUS) & MAILBOX_EMPTY != 0
                {
                    not_empty = false
                }

                // The program won't wait forever.
                count += 1;
                if count > 0x00100000 { return None; }
            }

            mmio::mem_barrier();
            let data = mmio::read(MAIL0_READ);
            mmio::mem_barrier();

            // Check it is the correct channel
            if (data & 0b1111) as u8 == channel
            {
                return Some(data & !0b1111);
            }
        }
    }
}

pub fn send<T>(channel: u8, data: *mut T)
{
    unsafe
    {
        let mut not_full = true;

        while not_full
        {
            mmio::mem_barrier();
            if mmio::read(MAIL0_STATUS) & MAILBOX_FULL != 0
            {
                not_full = false
            }
        }

        mmio::flush_cache();

        mmio::mem_barrier();
        mmio::write(MAIL0_WRITE, (data as u32) | (channel as u32));
    }
}