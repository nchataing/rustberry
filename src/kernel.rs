#![no_std]
#![feature(asm, lang_items)]
#![allow(dead_code)]

extern crate rlibc;
mod mmio;
mod gpio;
mod uart;
pub mod panic;

use uart::{Uart, Write};

#[no_mangle]
pub extern fn kernel_main(r0: u32, r1: u32, atags: u32) -> !
{
    uart::init();

    uart::write_str("Hello world !\n");
    write!(Uart, "r0 = {}\n", r0).unwrap();
    write!(Uart, "r1 = {}\n", r1).unwrap();
    write!(Uart, "atags = 0x{:x}\n", atags).unwrap();

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}
