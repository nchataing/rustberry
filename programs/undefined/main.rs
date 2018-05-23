#![no_std]
#![feature(asm)]

extern crate rustberry_std as std;

#[no_mangle]
pub unsafe extern fn main()
{
    asm!(".word 0xf7f0a000" :::: "volatile");
}
