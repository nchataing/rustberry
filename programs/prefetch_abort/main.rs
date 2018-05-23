#![no_std]
#![feature(asm)]

extern crate rustberry_std as std;

#[no_mangle]
pub unsafe extern fn main()
{
    asm!("bx $0" :: "r"(0) :: "volatile");
}
