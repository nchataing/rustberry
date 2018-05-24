#![no_std]
#![feature(asm)]

#[macro_use] extern crate rustberry_std as std;

#[no_mangle]
pub extern fn main()
{
    loop
    {
        let child_ev = std::syscall::wait_children();
        print!("Child {} ended with exit code {}", child_ev.pid, child_ev.exit_code);
    }
}
