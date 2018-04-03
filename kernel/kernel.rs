#![no_std]
#![feature(asm, lang_items, const_fn)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

extern crate rustberry_drivers as drivers;

#[macro_use] mod linker_symbol;

pub mod exceptions;
pub mod interrupts;
pub mod panic;
mod system_timer;
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
    mem::mmu::init();

    uart::init();
    write!(Uart, "Hello world !\n").unwrap();

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
        // The following operation must fail !
        mmio::write(0 as *mut u32, 0);
    }

    write!(Uart, "π = {}\n", core::f32::consts::PI).unwrap();

    let scr : u32;
    unsafe
    {
        asm!("mrc p15, 0, $0, c1, c1, 0" : "=r"(scr));
    }
    write!(Uart, "Secure mode : {:b}\n", scr).unwrap();

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}

