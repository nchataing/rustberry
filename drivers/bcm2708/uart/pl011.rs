use crate::gpio;
use crate::mmio;
use crate::CharacterDevice;
pub use core::fmt::{Result, Write};

/// The base address for UART.
const UART0_BASE: usize = (gpio::GPIO_BASE + 0x1000);

// The offsets for reach register for the UART.
const UART0_DR: *mut u32 = (UART0_BASE + 0x00) as *mut u32;
const UART0_RSRECR: *mut u32 = (UART0_BASE + 0x04) as *mut u32;
const UART0_FR: *mut u32 = (UART0_BASE + 0x18) as *mut u32;
const UART0_ILPR: *mut u32 = (UART0_BASE + 0x20) as *mut u32;
const UART0_IBRD: *mut u32 = (UART0_BASE + 0x24) as *mut u32;
const UART0_FBRD: *mut u32 = (UART0_BASE + 0x28) as *mut u32;
const UART0_LCRH: *mut u32 = (UART0_BASE + 0x2C) as *mut u32;
const UART0_CR: *mut u32 = (UART0_BASE + 0x30) as *mut u32;
const UART0_IFLS: *mut u32 = (UART0_BASE + 0x34) as *mut u32;
const UART0_IMSC: *mut u32 = (UART0_BASE + 0x38) as *mut u32;
const UART0_RIS: *mut u32 = (UART0_BASE + 0x3C) as *mut u32;
const UART0_MIS: *mut u32 = (UART0_BASE + 0x40) as *mut u32;
const UART0_ICR: *mut u32 = (UART0_BASE + 0x44) as *mut u32;
const UART0_DMACR: *mut u32 = (UART0_BASE + 0x48) as *mut u32;
const UART0_ITCR: *mut u32 = (UART0_BASE + 0x80) as *mut u32;
const UART0_ITIP: *mut u32 = (UART0_BASE + 0x84) as *mut u32;
const UART0_ITOP: *mut u32 = (UART0_BASE + 0x88) as *mut u32;
const UART0_TDR: *mut u32 = (UART0_BASE + 0x8C) as *mut u32;

// RSRECR bits
const RSRECR_OVERRUN: u32 = 1 << 3;

// FR bits
const FR_BUSY: u32 = 1 << 3;
const FR_RX_FIFO_EMPTY: u32 = 1 << 4;
const FR_TX_FIFO_FULL: u32 = 1 << 5;

// LCRH bits
const LCRH_ENABLE_FIFO: u32 = 1 << 4;
const LCRH_8BIT: u32 = 0b11 << 5;

// CR bits
const CR_UARTEN: u32 = 1 << 0;
const CR_TXEN: u32 = 1 << 8;
const CR_RXEN: u32 = 1 << 9;

// Interrupt bits
const CTSM_INT: u32 = 1 << 1;
const RX_INT: u32 = 1 << 4;
const TX_INT: u32 = 1 << 5;
const RCV_TIMEOUT_INT: u32 = 1 << 6;
const FRAMING_ERROR_INT: u32 = 1 << 7;
const PARITY_ERROR_INT: u32 = 1 << 8;
const BREAK_ERROR_INT: u32 = 1 << 9;
const OVERRUN_ERROR_INT: u32 = 1 << 10;
const ALL_INT: u32 = CTSM_INT
    | RX_INT
    | TX_INT
    | RCV_TIMEOUT_INT
    | FRAMING_ERROR_INT
    | PARITY_ERROR_INT
    | BREAK_ERROR_INT
    | OVERRUN_ERROR_INT;

const UART_CLOCK: u32 = 48_000_000;
const BAUD_RATE: u32 = 115_200;

/// Initialize the PL011 UART to use the pins 14 & 15 with baud rate 115200
pub fn init() {
    unsafe {
        // Disable UART0.
        mmio::write(UART0_CR, 0);

        // Setup the GPIO pin 14 & 15.
        gpio::select_pin_function(14, gpio::PinFunction::Alt0);
        gpio::select_pin_function(15, gpio::PinFunction::Alt0);

        // Disable pull up/down for pin 14 & 15.
        gpio::set_pull_mode(14, gpio::PullMode::Disabled);
        gpio::set_pull_mode(15, gpio::PullMode::Disabled);

        // Clear pending interrupts.
        mmio::write(UART0_ICR, ALL_INT);

        // Set integer & fractional part of baud rate.
        // Divider = UART_CLOCK/(16 * Baud)
        // Fraction part register = (Fractional part * 64) + 0.5
        let quot = (4 * UART_CLOCK + (BAUD_RATE / 2)) / BAUD_RATE;
        mmio::write(UART0_IBRD, quot >> 6);
        mmio::write(UART0_FBRD, quot & 0x3f);

        // Enable FIFO & 8 bit data transmission (1 stop bit, no parity).
        mmio::write(UART0_LCRH, LCRH_ENABLE_FIFO | LCRH_8BIT);

        // Mask all interrupts.
        mmio::write(UART0_IMSC, ALL_INT);

        // Enable UART0, receive & transfer part of UART.
        mmio::write(UART0_CR, CR_UARTEN | CR_TXEN | CR_RXEN);
    }
}

/// Read one byte on the UART. Wait until it is available (blocking IO).
pub fn read_byte() -> u8 {
    // Wait for UART to have received something.
    while !has_char_available() {}

    unsafe { mmio::read(UART0_DR) as u8 }
}

/// Return true if there is some data available in the recieve FIFO.
pub fn has_char_available() -> bool {
    unsafe { mmio::read(UART0_FR) & FR_RX_FIFO_EMPTY == 0 }
}

/// Return true if there was a reciever overrun since the last UART operation.
pub fn got_overrun() -> bool {
    unsafe { mmio::read(UART0_RSRECR) & RSRECR_OVERRUN != 0 }
}

/// Write one byte in the UART transmit FIFO. Wait if necessary.
pub fn write_byte(c: u8) {
    unsafe {
        // Wait for UART to become ready to transmit.
        while (mmio::read(UART0_FR) & FR_TX_FIFO_FULL) != 0 {}
        mmio::write(UART0_DR, c as u32);
    }
}

/// Write the string on the UART.
pub fn write_str(s: &str) {
    for c in s.bytes() {
        write_byte(c);
    }
}

/// Wait until all the data has been written and transmit FIFO is empty.
pub fn flush() {
    unsafe { while mmio::read(UART0_FR) & FR_BUSY != 0 {} }
}

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result {
        write_str(s);
        Ok(())
    }
}

impl CharacterDevice for Uart {
    fn read_byte(&self) -> u8 {
        read_byte()
    }

    fn write_byte(&self, c: u8) {
        write_byte(c)
    }

    fn flush(&self) {
        flush()
    }
}
