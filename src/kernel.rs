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
pub extern fn kernel_main(r0: u32, r1: u32, r2: u32) -> !
{
    uart::init();
    write!(Uart0, "Hello world !\n");

    write!(Uart0, "r0 = {}, r1 = {}, r2 = {}\n", r0, r1, r2);
    write!(Uart0, "FPU ! {} / {} = {}\n", 4., 5., (4.+(r0 as f32))/5.);

    panic!("Ahah !");

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}
