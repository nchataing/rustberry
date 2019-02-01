#![no_std]
#![feature(asm)]

#[macro_use]
extern crate rustberry_std as std;

#[no_mangle]
pub extern "C" fn main() {
    loop {
        std::syscall::sleep(1000);
        print!("!")
    }
}
