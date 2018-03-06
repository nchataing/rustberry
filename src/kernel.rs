#![no_std]
#![feature(asm, lang_items)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

mod mmio;
mod gpio;
mod uart;
pub mod panic;

use uart::{Uart0, Write};

#[no_mangle]
pub extern fn kernel_main() -> !
{
    uart::init();
    uart::write_str("Hello world !\n");

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}
