#![no_std]
#![feature(asm, lang_items)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

extern crate rustberry_drivers as drivers;

pub mod exceptions;
pub mod interrupts;
pub mod panic;
mod system_timer;

use drivers::*;
use drivers::uart::{Uart, Write};

fn timer1_handler()
{
    system_timer::clear_irq(system_timer::Timer1);
    write!(Uart, "1").unwrap();
    system_timer::set_remaining_time(system_timer::Timer1, 1_000_000);
}

fn timer3_handler()
{
    system_timer::clear_irq(system_timer::Timer3);
    write!(Uart, "3").unwrap();
    system_timer::set_remaining_time(system_timer::Timer3, 3_000_000);
}


#[no_mangle]
pub extern fn kernel_main() -> !
{
    uart::init();
    write!(Uart, "Hello world !\n").unwrap();

    interrupts::init();
    interrupts::register_fiq(1, timer1_handler);
    system_timer::register_callback(system_timer::Timer3, timer3_handler);
    system_timer::set_remaining_time(system_timer::Timer1, 1_000_000);
    system_timer::set_remaining_time(system_timer::Timer3, 1_000_000);

    unsafe
    {
        asm!("svc 42" ::: "r0","r1","r2","r3","r12","lr","cc" : "volatile");
    }

    write!(Uart, "π = {}\n", core::f32::consts::PI).unwrap();

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}

