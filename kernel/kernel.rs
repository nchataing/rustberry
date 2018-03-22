#![no_std]
#![feature(asm, lang_items)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

extern crate rustberry_drivers as drivers;

pub mod exceptions;
pub mod panic;

use drivers::*;
use drivers::uart::{Uart, Write};

#[no_mangle]
pub extern fn kernel_main() -> !
{
    uart::init();
    write!(Uart, "Hello world !\n").unwrap();

    unsafe
    {
        asm!("svc 42" ::: "r0","r1","r2","r3","r12","lr","cc" : "volatile");
    }

    write!(Uart, "π = {}\n", core::f32::consts::PI).unwrap();

    let fb_data = framebuffer::init(640,480);
    write!(Uart, "{:?}\n", fb_data).unwrap();

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}

