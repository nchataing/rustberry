#![no_std]
#![feature(asm, lang_items)]

extern crate rlibc;

extern
{
    fn main();
}

#[no_mangle]
pub extern fn start()
{
    unsafe { main(); }
}

use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(_msg: fmt::Arguments, _file: &'static str,
                        _line: u32, _column: u32) -> !
{
    // TODO: Implement this
    loop {}
}
