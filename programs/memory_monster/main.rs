#![no_std]
#![feature(alloc)]
#[macro_use]
extern crate alloc;
extern crate rustberry_std as std;

#[no_mangle]
pub extern "C" fn main() {
    let mut a = vec![];
    loop {
        a.push(42);
    }
}
