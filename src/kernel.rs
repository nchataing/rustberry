#![no_std]
#![feature(asm, lang_items)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

mod mmio;
mod gpio;
mod mini_uart;
mod mailbox;
mod framebuffer;
pub mod panic;

use mini_uart::{Uart1, Write};

#[no_mangle]
pub extern fn kernel_main(dummy : i32) -> !
{
    mini_uart::init();
    write!(Uart1, "Hello world !\n").unwrap();

    let fb_data = framebuffer::init(640,480);
    write!(Uart1, "{:?}\n", fb_data).unwrap();

    write!(Uart1, "42 = {}\n", 40+2).unwrap();
    write!(Uart1, "FPU ! {} / {} = {}\n", dummy as f32, 5., (dummy as f32)/5.).unwrap();
    panic!("Ahah !");

    loop
    {
        let c = mini_uart::read_byte();
        mini_uart::write_byte(c);
    }
}
