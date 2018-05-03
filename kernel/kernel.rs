#![no_std]
#![feature(asm, lang_items, const_fn, iterator_step_by)]
#![allow(dead_code)]

#![feature(alloc, allocator_api, global_allocator)]
#[macro_use] extern crate alloc;
extern crate rlibc;

#[macro_use] extern crate bitflags;

#[macro_use] extern crate rustberry_drivers as drivers;

#[macro_use] mod linker_symbol;
pub mod exceptions;
pub mod panic;
mod system_control;
mod atag;
pub mod mem;

use drivers::*;

use mem::kernel_alloc::GlobalKernelAllocator;
#[global_allocator]
static ALLOCATOR: GlobalKernelAllocator = GlobalKernelAllocator;

fn timer_handler()
{
    print!(".");
    core_timer::set_remaining_time(core_timer::Physical, 10_000_000);
}

#[no_mangle]
pub extern fn kernel_main() -> !
{
    mem::map::init();

    uart::init();
    println!("\x1b[32;1mHello world !\x1b[0m");

    let size = atag::get_mem_size();
    println!("Memory size: {:#x}", size);

    interrupts::init();

    if let Ok(sdcard) = emmc::init()
    {
        let mut first_sdblock = [0; emmc::BLOCK_SIZE];
        sdcard.read(&mut first_sdblock, 0).unwrap();
        println!("First SD card block:");
        for chunk in first_sdblock.chunks(16)
        {
            for val in chunk
            {
                print!("{:02x}, ", val);
            }
            print!("\n");
        }
    }

    core_timer::init();
    core_timer::register_callback(core_timer::Physical, timer_handler, false);
    core_timer::set_enabled(core_timer::Physical, true);
    core_timer::set_remaining_time(core_timer::Physical, 10_000_000);

    unsafe
    {
        asm!("svc 42" ::: "r0","r1","r2","r3","r12","lr","cc" : "volatile");
    }

    mem::physical_alloc::init();

    /*unsafe
    {
        // Each of the following operations must fail !
        mmio::write(0 as *mut u32, 0); // Data abort
        asm!("bx $0" :: "r"(0x2000) :: "volatile"); // Prefetch abort
    }*/

    println!("π = {}", core::f32::consts::PI);

    /*random::init();
    println!("Random -> {:#08x}", random::generate());*/
    unsafe
    {
        //use core::alloc::{Layout, GlobalAlloc};
        //let addr = ALLOCATOR.alloc(Layout::from_size_align_unchecked(15,4)) as *mut u32;
        /*for i in 0x00 .. 0x20
        {
            println!("{:#x}", *((0x6000_0000 + 4*i) as *mut u32));
        }
        println!("");*/

        let v1 = vec![1337;0x1337];
        for i in 0x00 .. 0x20
        {
            println!("{:#x}", *((0x6000_0000 + 4*i) as *mut u32));
        }
        println!("\n{}\n", v1[2]);
        drop(v1);
    }

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}

