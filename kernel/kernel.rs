#![no_std]
#![feature(asm, lang_items, const_fn)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;
#[macro_use] extern crate bitflags;

#[macro_use] extern crate rustberry_drivers as drivers;

#[macro_use] mod linker_symbol;
pub mod exceptions;
pub mod panic;
mod system_control;
mod atag;
mod mem;

use drivers::*;
use drivers::uart::{Uart, Write};

fn timer_handler()
{
    write!(Uart, ".").unwrap();
    core_timer::set_remaining_time(core_timer::Physical, 10_000_000);
}

#[no_mangle]
pub extern fn kernel_main() -> !
{
    mem::map::init();

    uart::init();
    write!(Uart, "\x1b[32;1mHello world !\x1b[0m\n").unwrap();

    let size = atag::get_mem_size();
    write!(Uart, "Memory size: {:#x}\n", size).unwrap();

    interrupts::init();
    core_timer::init();
    core_timer::register_callback(core_timer::Physical, timer_handler, false);
    core_timer::set_enabled(core_timer::Physical, true);
    core_timer::set_remaining_time(core_timer::Physical, 10_000_000);

    unsafe
    {
        asm!("svc 42" ::: "r0","r1","r2","r3","r12","lr","cc" : "volatile");
    }

    mem::pages::init();
    let test_page = mem::pages::allocate_page();
    mem::pages::deallocate_page(test_page);

    /*unsafe
    {
        // Each of the following operations must fail !
        mmio::write(0 as *mut u32, 0); // Data abort
        asm!("bx $0" :: "r"(0x2000) :: "volatile"); // Prefetch abort
    }*/

    write!(Uart, "π = {}\n", core::f32::consts::PI).unwrap();

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}

