use mmio;
use gpio;
pub use core::fmt::{Write, Result};

const AUX_BASE : usize = mmio::PERIPHERAL_BASE + 0x215000;

const AUX_ENABLES     : *mut u32 = (AUX_BASE + 0x04) as *mut u32;
const AUX_MU_IO_REG   : *mut u32 = (AUX_BASE + 0x40) as *mut u32;
const AUX_MU_IER_REG  : *mut u32 = (AUX_BASE + 0x44) as *mut u32;
const AUX_MU_IIR_REG  : *mut u32 = (AUX_BASE + 0x48) as *mut u32;
const AUX_MU_LCR_REG  : *mut u32 = (AUX_BASE + 0x4C) as *mut u32;
const AUX_MU_MCR_REG  : *mut u32 = (AUX_BASE + 0x50) as *mut u32;
const AUX_MU_LSR_REG  : *mut u32 = (AUX_BASE + 0x54) as *mut u32;
const AUX_MU_MSR_REG  : *mut u32 = (AUX_BASE + 0x58) as *mut u32;
const AUX_MU_SCRATCH  : *mut u32 = (AUX_BASE + 0x5C) as *mut u32;
const AUX_MU_CNTL_REG : *mut u32 = (AUX_BASE + 0x60) as *mut u32;
const AUX_MU_STAT_REG : *mut u32 = (AUX_BASE + 0x64) as *mut u32;
const AUX_MU_BAUD_REG : *mut u32 = (AUX_BASE + 0x68) as *mut u32;

pub fn init()
{
    unsafe
    {
        gpio::select_pin_function(14, gpio::PinFunction::Alt5);
        gpio::select_pin_function(15, gpio::PinFunction::Alt5);

        gpio::set_pull_up_down(14, gpio::PullUpDownMode::Disabled);
        gpio::set_pull_up_down(15, gpio::PullUpDownMode::Disabled);

        mmio::write(AUX_ENABLES,1);
        mmio::write(AUX_MU_IER_REG,0);
        mmio::write(AUX_MU_CNTL_REG,0);
        mmio::write(AUX_MU_LCR_REG,3);
        mmio::write(AUX_MU_MCR_REG,0);
        mmio::write(AUX_MU_IER_REG,0);
        mmio::write(AUX_MU_IIR_REG,0xC6);
        mmio::write(AUX_MU_BAUD_REG,270);
        mmio::write(AUX_MU_CNTL_REG,3);
    }
}

pub fn read_byte() -> u8
{
    unsafe
    {
        while !has_char_available() {}
        (mmio::read(AUX_MU_IO_REG) & 0xFF) as u8
    }
}

pub fn has_char_available() -> bool
{
    unsafe
    {
        mmio::read(AUX_MU_LSR_REG) & 0x01 != 0
    }
}

pub fn write_byte(c : u8)
{
    unsafe
    {
        while mmio::read(AUX_MU_LSR_REG) & 0x20 == 0 {}
        mmio::write(AUX_MU_IO_REG, c as u32);
    }
}

pub fn write_str(s: &str)
{
    for c in s.bytes()
    {
        write_byte(c);
    }
}

/*pub fn flush()
{
    unsafe
    {
        while mmio::read(AUX_MU_LSR_REG) & 0x100 != 0 {}
    }
}*/

pub struct Uart1;

impl Write for Uart1
{
    fn write_str(&mut self, s: &str) -> Result
    {
        write_str(s);
        Ok(())
    }
}
