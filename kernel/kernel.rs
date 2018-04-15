#![no_std]
#![feature(asm, lang_items, const_fn)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

#[macro_use]
extern crate bitflags;

extern crate rustberry_drivers as drivers;

#[macro_use] mod linker_symbol;
#[macro_use] mod coproc_reg;

pub mod exceptions;
pub mod interrupts;
pub mod panic;
mod system_timer;
mod system_control;
mod atag;
mod mem;

use drivers::*;
use drivers::uart::{Uart, Write};

fn timer_handler()
{
    system_timer::clear_irq(system_timer::Timer1);
    write!(Uart, ".").unwrap();
    system_timer::set_remaining_time(system_timer::Timer1, 1_000_000);
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
    system_timer::register_callback(system_timer::Timer1, timer_handler);
    system_timer::set_remaining_time(system_timer::Timer1, 1_000_000);

    unsafe
    {
        asm!("svc 42" ::: "r0","r1","r2","r3","r12","lr","cc" : "volatile");
    }

    mem::pages::init();
    let test_page = mem::pages::allocate_page();
    mem::pages::deallocate_page(test_page);

    unsafe
    {
        // Each of the following operations must fail !
        mmio::write(0 as *mut u32, 0); // Data abort
        asm!("bx $0" :: "r"(0x2000) :: "volatile"); // Prefetch abort
    }

    write!(Uart, "π = {}\n", core::f32::consts::PI).unwrap();

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}

