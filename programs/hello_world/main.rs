#![no_std]

extern crate rustberry_io as io;
#[macro_use]
extern crate rustberry_std as std;

#[no_mangle]
pub extern "C" fn main() {
    print!("Hello application world !");
}
