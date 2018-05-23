#![no_std]
#![feature(asm)]

extern crate rustberry_std as std;

use core::ptr;

#[no_mangle]
pub unsafe extern fn main()
{
    ptr::read_volatile(0x0 as *const u32);
}
