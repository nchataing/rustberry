use bcm2708;
use bcm2708::gpio;
pub use core::fmt::{Result, Write};
use mmio;

const AUX_BASE: usize = bcm2708::PERIPHERAL_BASE + 0x215000;

const AUX_ENABLES: *mut u32 = (AUX_BASE + 0x04) as *mut u32;
const AUX_MU_IO_REG: *mut u32 = (AUX_BASE + 0x40) as *mut u32;
const AUX_MU_IER_REG: *mut u32 = (AUX_BASE + 0x44) as *mut u32;
const AUX_MU_IIR_REG: *mut u32 = (AUX_BASE + 0x48) as *mut u32;
const AUX_MU_LCR_REG: *mut u32 = (AUX_BASE + 0x4C) as *mut u32;
const AUX_MU_MCR_REG: *mut u32 = (AUX_BASE + 0x50) as *mut u32;
const AUX_MU_LSR_REG: *mut u32 = (AUX_BASE + 0x54) as *mut u32;
const AUX_MU_MSR_REG: *mut u32 = (AUX_BASE + 0x58) as *mut u32;
const AUX_MU_SCRATCH: *mut u32 = (AUX_BASE + 0x5C) as *mut u32;
const AUX_MU_CNTL_REG: *mut u32 = (AUX_BASE + 0x60) as *mut u32;
const AUX_MU_STAT_REG: *mut u32 = (AUX_BASE + 0x64) as *mut u32;
const AUX_MU_BAUD_REG: *mut u32 = (AUX_BASE + 0x68) as *mut u32;

// AUX_MU_LCR bits
const DATA_READY: u32 = 1 << 0;
const RX_OVERRUN: u32 = 1 << 1;
const TX_NOT_FULL: u32 = 1 << 5;
const TX_IDLE: u32 = 1 << 6;

/// Initialize the Mini UART to use the pins 14 & 15 with baud rate 115200
pub fn init() {
    unsafe {
        gpio::select_pin_function(14, gpio::PinFunction::Alt5);
        gpio::select_pin_function(15, gpio::PinFunction::Alt5);

        gpio::set_pull_mode(14, gpio::PullMode::Disabled);
        gpio::set_pull_mode(15, gpio::PullMode::Disabled);

        mmio::write(AUX_ENABLES, 1);
        mmio::write(AUX_MU_IER_REG, 0);
        mmio::write(AUX_MU_CNTL_REG, 0);
        mmio::write(AUX_MU_LCR_REG, 3);
        mmio::write(AUX_MU_MCR_REG, 0);
        mmio::write(AUX_MU_IER_REG, 0);
        mmio::write(AUX_MU_IIR_REG, 0xC6);
        mmio::write(AUX_MU_BAUD_REG, 270);
        mmio::write(AUX_MU_CNTL_REG, 3);
    }
}

/// Return true if there is some data available in the recieve FIFO.
pub fn has_char_available() -> bool {
    unsafe { mmio::read(AUX_MU_LSR_REG) & DATA_READY != 0 }
}

/// Return true if there was a reciever overrun since the last UART operation.
pub fn got_overrun() -> bool {
    unsafe { mmio::read(AUX_MU_LSR_REG) & RX_OVERRUN != 0 }
}

/// Write one byte in the UART transmit FIFO. Wait if necessary.
pub fn write_byte(c: u8) {
    unsafe {
        while mmio::read(AUX_MU_LSR_REG) & TX_NOT_FULL == 0 {}
        mmio::write(AUX_MU_IO_REG, c as u32);
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
    unsafe { while mmio::read(AUX_MU_LSR_REG) & TX_IDLE == 0 {} }
}

pub struct Uart;

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result {
        write_str(s);
        Ok(())
    }
}
