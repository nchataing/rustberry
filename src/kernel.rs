#![no_std]
#![feature(asm, lang_items)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

mod mmio;
mod gpio;
mod uart;
mod mailbox;
mod framebuffer;
pub mod panic;

use uart::{Uart, Write};

#[no_mangle]
pub extern fn kernel_main(dummy : i32) -> !
{
    uart::init();
    write!(Uart, "Hello world !\n").unwrap();

    let fb_data = framebuffer::init(640,480);
    write!(Uart, "{:?}\n", fb_data).unwrap();

    write!(Uart, "42 = {}\n", 40+2).unwrap();
    //write!(Uart, "FPU ! {} / {} = {}\n", dummy as f32, 5., (dummy as f32)/5.).unwrap();
    panic!("Ahah !");

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}
