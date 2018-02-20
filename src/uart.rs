use mmio;

/// The base address for UART.
const UART0_BASE : usize = (mmio::GPIO_BASE + 0x1000);

// The offsets for reach register for the UART.
const UART0_DR     : *mut i32 = (UART0_BASE + 0x00) as *mut i32;
//const UART0_RSRECR : *mut i32 = (UART0_BASE + 0x04) as *mut i32;
const UART0_FR     : *mut i32 = (UART0_BASE + 0x18) as *mut i32;
//const UART0_ILPR   : *mut i32 = (UART0_BASE + 0x20) as *mut i32;
const UART0_IBRD   : *mut i32 = (UART0_BASE + 0x24) as *mut i32;
const UART0_FBRD   : *mut i32 = (UART0_BASE + 0x28) as *mut i32;
const UART0_LCRH   : *mut i32 = (UART0_BASE + 0x2C) as *mut i32;
const UART0_CR     : *mut i32 = (UART0_BASE + 0x30) as *mut i32;
//const UART0_IFLS   : *mut i32 = (UART0_BASE + 0x34) as *mut i32;
const UART0_IMSC   : *mut i32 = (UART0_BASE + 0x38) as *mut i32;
//const UART0_RIS    : *mut i32 = (UART0_BASE + 0x3C) as *mut i32;
//const UART0_MIS    : *mut i32 = (UART0_BASE + 0x40) as *mut i32;
const UART0_ICR    : *mut i32 = (UART0_BASE + 0x44) as *mut i32;
//const UART0_DMACR  : *mut i32 = (UART0_BASE + 0x48) as *mut i32;
//const UART0_ITCR   : *mut i32 = (UART0_BASE + 0x80) as *mut i32;
//const UART0_ITIP   : *mut i32 = (UART0_BASE + 0x84) as *mut i32;
//const UART0_ITOP   : *mut i32 = (UART0_BASE + 0x88) as *mut i32;
//const UART0_TDR    : *mut i32 = (UART0_BASE + 0x8C) as *mut i32;

pub fn init()
{
    unsafe
    {
        // Disable UART0.
        mmio::write(UART0_CR, 0x00000000);
        // Setup the GPIO pin 14 && 15.

        // Disable pull up/down for all GPIO pins & delay for 150 cycles.
        mmio::write(mmio::GPPUD, 0x00000000);
        mmio::delay(150);

        // Disable pull up/down for pin 14,15 & delay for 150 cycles.
        mmio::write(mmio::GPPUDCLK0, (1 << 14) | (1 << 15));
        mmio::delay(150);

        // Write 0 to GPPUDCLK0 to make it take effect.
        mmio::write(mmio::GPPUDCLK0, 0x00000000);

        // Clear pending interrupts.
        mmio::write(UART0_ICR, 0x7FF);

        // Set integer & fractional part of baud rate.
        // Divider = UART_CLOCK/(16 * Baud)
        // Fraction part register = (Fractional part * 64) + 0.5
        // UART_CLOCK = 3000000; Baud = 115200.

        // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
        mmio::write(UART0_IBRD, 1);
        // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
        mmio::write(UART0_FBRD, 40);

        // Enable FIFO & 8 bit data transmissio (1 stop bit, no parity).
        mmio::write(UART0_LCRH, (1 << 4) | (1 << 5) | (1 << 6));

        // Mask all interrupts.
        mmio::write(UART0_IMSC, (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
                            (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10));

        // Enable UART0, receive & transfer part of UART.
        mmio::write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
    }
}

pub fn putc(c: u8)
{
    unsafe
    {
        // Wait for UART to become ready to transmit.
        while (mmio::read(UART0_FR) & (1 << 5)) != 0 { }
        mmio::write(UART0_DR, c as i32);
    }
}

pub fn getc() -> u8
{
    unsafe
    {
        // Wait for UART to have received something.
        while (mmio::read(UART0_FR) & (1 << 4)) != 0 { }
        mmio::read(UART0_DR) as u8
    }
}

pub fn puts(s: &str)
{
    for c in s.bytes()
    {
        putc(c);
    }
}
