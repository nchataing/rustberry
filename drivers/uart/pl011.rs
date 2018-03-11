use mmio;
use gpio;
pub use core::fmt::{Write, Result};

/// The base address for UART.
const UART0_BASE : usize = (gpio::GPIO_BASE + 0x1000);

// The offsets for reach register for the UART.
const UART0_DR     : *mut u32 = (UART0_BASE + 0x00) as *mut u32;
const UART0_RSRECR : *mut u32 = (UART0_BASE + 0x04) as *mut u32;
const UART0_FR     : *mut u32 = (UART0_BASE + 0x18) as *mut u32;
const UART0_ILPR   : *mut u32 = (UART0_BASE + 0x20) as *mut u32;
const UART0_IBRD   : *mut u32 = (UART0_BASE + 0x24) as *mut u32;
const UART0_FBRD   : *mut u32 = (UART0_BASE + 0x28) as *mut u32;
const UART0_LCRH   : *mut u32 = (UART0_BASE + 0x2C) as *mut u32;
const UART0_CR     : *mut u32 = (UART0_BASE + 0x30) as *mut u32;
const UART0_IFLS   : *mut u32 = (UART0_BASE + 0x34) as *mut u32;
const UART0_IMSC   : *mut u32 = (UART0_BASE + 0x38) as *mut u32;
const UART0_RIS    : *mut u32 = (UART0_BASE + 0x3C) as *mut u32;
const UART0_MIS    : *mut u32 = (UART0_BASE + 0x40) as *mut u32;
const UART0_ICR    : *mut u32 = (UART0_BASE + 0x44) as *mut u32;
const UART0_DMACR  : *mut u32 = (UART0_BASE + 0x48) as *mut u32;
const UART0_ITCR   : *mut u32 = (UART0_BASE + 0x80) as *mut u32;
const UART0_ITIP   : *mut u32 = (UART0_BASE + 0x84) as *mut u32;
const UART0_ITOP   : *mut u32 = (UART0_BASE + 0x88) as *mut u32;
const UART0_TDR    : *mut u32 = (UART0_BASE + 0x8C) as *mut u32;

pub fn init()
{
    unsafe
    {
        // Disable UART0.
        mmio::write(UART0_CR, 0x00000000);

        // Setup the GPIO pin 14 & 15.
        gpio::select_pin_function(14, gpio::PinFunction::Alt0);
        gpio::select_pin_function(15, gpio::PinFunction::Alt0);

        // Disable pull up/down for pin 14 & 15.
        gpio::set_pull_up_down(14, gpio::PullUpDownMode::Disabled);
        gpio::set_pull_up_down(15, gpio::PullUpDownMode::Disabled);

        // Clear pending interrupts.
        mmio::write(UART0_ICR, (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
                            (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10));

        // Set integer & fractional part of baud rate.
        // Divider = UART_CLOCK/(16 * Baud)
        // Fraction part register = (Fractional part * 64) + 0.5
        // UART_CLOCK = 3000000; Baud = 115200.

        // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
        mmio::write(UART0_IBRD, 1);
        // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
        mmio::write(UART0_FBRD, 40);

        // Enable FIFO & 8 bit data transmission (1 stop bit, no parity).
        mmio::write(UART0_LCRH, (1 << 4) | (0b11 << 5));

        // Mask all interrupts.
        mmio::write(UART0_IMSC, (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
                            (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10));

        // Enable UART0, receive & transfer part of UART.
        mmio::write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
    }
}

pub fn read_byte() -> u8
{
    // Wait for UART to have received something.
    while !has_char_available() { }

    unsafe
    {
        mmio::read(UART0_DR) as u8
    }

}

pub fn has_char_available() -> bool
{
    unsafe
    {
        mmio::read(UART0_FR) & (1 << 4) == 0
    }
}

pub fn write_byte(c: u8)
{
    unsafe
    {
        // Wait for UART to become ready to transmit.
        while (mmio::read(UART0_FR) & (1 << 5)) != 0 { }
        mmio::write(UART0_DR, c as u32);
    }
}

pub fn write_str(s: &str)
{
    for c in s.bytes()
    {
        write_byte(c);
    }
}

pub struct Uart;

impl Write for Uart
{
    fn write_str(&mut self, s: &str) -> Result
    {
        write_str(s);
        Ok(())
    }
}

