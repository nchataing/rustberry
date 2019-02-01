#![no_std]
#![feature(asm)]
#![allow(dead_code)]

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod coproc_reg;
#[macro_use]
pub mod mmio;
mod bcm2708;
mod quad_a7;

pub use bcm2708::{emmc, gpio, random, system_timer, uart, video_core};
pub use quad_a7::{core_timer, get_core_id, interrupts, mailbox};

pub trait CharacterDevice {
    fn read_byte(&self) -> u8;
    fn write_byte(&self, c: u8);
    fn flush(&self);
}

#[macro_export]
macro_rules! print
{
    ($($arg:tt)*) =>
    {{
        use $crate::uart::{Uart, Write};
        let _ = write!(Uart, $($arg)*);
    }}
}

#[macro_export]
macro_rules! println
{
    ($($arg:tt)*) =>
    {{
        use $crate::uart::{Uart, Write};
        let _ = writeln!(Uart, $($arg)*);
    }}
}
