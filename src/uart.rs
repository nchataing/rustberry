use mmio;
use gpio;
pub use core::fmt::{Write, Result};

const AUX_ENABLES     : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215004) as *mut i32;
const AUX_MU_IO_REG   : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215040) as *mut i32;
const AUX_MU_IER_REG  : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215044) as *mut i32;
const AUX_MU_IIR_REG  : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215048) as *mut i32;
const AUX_MU_LCR_REG  : *mut i32 = (mmio::PERIPHERAL_BASE+0x0021504C) as *mut i32;
const AUX_MU_MCR_REG  : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215050) as *mut i32;
const AUX_MU_LSR_REG  : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215054) as *mut i32;
const AUX_MU_MSR_REG  : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215058) as *mut i32;
const AUX_MU_SCRATCH  : *mut i32 = (mmio::PERIPHERAL_BASE+0x0021505C) as *mut i32;
const AUX_MU_CNTL_REG : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215060) as *mut i32;
const AUX_MU_STAT_REG : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215064) as *mut i32;
const AUX_MU_BAUD_REG : *mut i32 = (mmio::PERIPHERAL_BASE+0x00215068) as *mut i32;

pub struct Uart;

impl Uart
{
    pub fn init() -> Uart
    {
        unsafe
        {
            mmio::write(AUX_ENABLES,1);
            mmio::write(AUX_MU_IER_REG,0);
            mmio::write(AUX_MU_CNTL_REG,0);
            mmio::write(AUX_MU_LCR_REG,3);
            mmio::write(AUX_MU_MCR_REG,0);
            mmio::write(AUX_MU_IER_REG,0);
            mmio::write(AUX_MU_IIR_REG,0xC6);
            mmio::write(AUX_MU_BAUD_REG,270);

            let mut ra = mmio::read(gpio::GPFSEL1);
            ra &= !(7<<12); //gpio14
            ra |= 2<<12;    //alt5
            ra &= !(7<<15); //gpio15
            ra |= 2<<15;    //alt5
            mmio::write(gpio::GPFSEL1,ra);

            mmio::write(gpio::GPPUD,0);
            mmio::delay(150);

            mmio::write(gpio::GPPUDCLK0,(1<<14)|(1<<15));
            mmio::delay(150);

            mmio::write(gpio::GPPUDCLK0,0);
            mmio::write(AUX_MU_CNTL_REG,3);
        }

        Uart
    }

    pub fn read_byte(&mut self) -> u8
    {
        unsafe
        {
            while !self.has_char_available() {}
            (mmio::read(AUX_MU_IO_REG) & 0xFF) as u8
        }
    }

    pub fn has_char_available(&mut self) -> bool
    {
        unsafe
        {
            mmio::read(AUX_MU_LSR_REG) & 0x01 != 0
        }
    }

    pub fn write_byte(&mut self, c : u8)
    {
        unsafe
        {
            while mmio::read(AUX_MU_LSR_REG) & 0x20 == 0 {}
            mmio::write(AUX_MU_IO_REG, c as i32);
        }
    }

    /*pub fn flush(&mut self)
    {
        unsafe
        {
            while mmio::read(AUX_MU_LSR_REG) & 0x100 != 0 {}
        }
    }*/
}

impl Write for Uart
{
    fn write_str(&mut self, s: &str) -> Result
    {
        for c in s.bytes()
        {
            self.write_byte(c);
        }

        Ok(())
    }
}
