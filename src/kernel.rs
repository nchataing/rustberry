#![no_std]
#![feature(asm, lang_items)]

extern crate rlibc;
mod mmio;
mod gpio;
mod uart;
pub mod panic;

use uart::{Uart, Write};

#[no_mangle]
pub extern fn kernel_main(r0: i32, r1: i32, atags: i32)
{
    let mut uart = Uart::init();
    uart.write_str("Hello world !");
    write!(&mut uart, "r0 = {}\n", r0).unwrap();
    write!(&mut uart, "r1 = {}\n", r1).unwrap();
    write!(&mut uart, "atags = 0x{:x}\n", atags).unwrap();

    loop
    {
        let c = uart.read_byte();
        uart.write_byte(c);
    }
}
