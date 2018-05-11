#![no_std]
#![feature(asm, lang_items, const_fn, iterator_step_by)]
#![allow(dead_code)]

#![feature(alloc, allocator_api, global_allocator)]
#[macro_use] extern crate alloc;
extern crate rlibc;

#[macro_use] extern crate bitflags;

#[macro_use] extern crate rustberry_drivers as drivers;
extern crate rustberry_allocator as allocator;

#[macro_use] mod log;
#[macro_use] mod linker_symbol;
pub mod exceptions;
pub mod panic;
mod system_control;
mod atag;
pub mod memory;
mod process;
mod filesystem;

use drivers::*;

use memory::kernel_alloc::GlobalKernelAllocator;
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
    memory::kernel_map::init();

    uart::init();
    println!("\x1b[32;1mHello world !\x1b[0m");

    let size = atag::get_mem_size();
    info!("Memory size: {:#x}", size);

    memory::physical_alloc::init();

    let v1 = vec![1337;42];
    println!("Dynamic allocation: 1337 = {}", v1[2]);
    drop(v1);

    interrupts::init();

    match emmc::init()
    {
        Ok(sdcard) =>
        {
            /*let mut first_sdblock = [0; emmc::BLOCK_SIZE];
            sdcard.read(&mut first_sdblock, 0).unwrap();
            println!("First SD card block:");
            for chunk in first_sdblock.chunks(16)
            {
                for val in chunk
                {
                    print!("{:02x}, ", val);
                }
                print!("\n");
            }*/

            match filesystem::mbr_reader::read_partition_table(sdcard)
            {
                Ok(partition_table) =>
                {
                    for partition in partition_table.iter()
                    {
                        println!("{:?}", partition);
                    }
                },
                Err(err) => warn!("MBR read failure: {:?}", err)
            }
        },
        Err(err) => warn!("SD card failure: {:?}", err)
    }

    core_timer::init();
    core_timer::register_callback(core_timer::Physical, timer_handler, false);
    core_timer::set_enabled(core_timer::Physical, true);
    core_timer::set_remaining_time(core_timer::Physical, 10_000_000);

    unsafe
    {
        asm!("svc 42" ::: "r0","r1","r2","r3","r12","lr","cc" : "volatile");
    }

    /*unsafe
    {
        // Each of the following operations must fail !
        mmio::write(0 as *mut u32, 0); // Data abort
        asm!("bx $0" :: "r"(0x2000) :: "volatile"); // Prefetch abort

        let page = memory::kernel_map::reserve_heap_pages(1);
        mmio::write(page.to_addr() as *mut u32, 42);
        memory::kernel_map::free_heap_pages(1);
        println!("{}", mmio::read(page.to_addr() as *mut u32));
    }*/

    unsafe
    {
        use alloc::boxed::Box;

        let mut appmap1 = Box::new(memory::application_map::ApplicationMap::new());
        appmap1.activate();
        let page1 = appmap1.reserve_heap_pages(1).unwrap();
        mmio::write(page1.to_addr() as *mut u32, 42);
        println!("{} @ {:p}", mmio::read(page1.to_addr() as *mut u32), page1.to_addr());

        mmio::instr_barrier();

        let mut appmap2 = Box::new(memory::application_map::ApplicationMap::new());
        let page2 = appmap2.reserve_heap_pages(1).unwrap();
        appmap2.activate();
        println!("{} @ {:p}", mmio::read(page2.to_addr() as *mut u32), page2.to_addr());
        mmio::write(page2.to_addr() as *mut u32, 54);
        println!("{} @ {:p}", mmio::read(page2.to_addr() as *mut u32), page2.to_addr());
    }

    println!("π = {}", core::f32::consts::PI);

    random::init();
    match random::generate()
    {
        Some(rand) => println!("Random -> {:#08x}", rand),
        None => warn!("Random engine timeout")
    }

    loop
    {
        let c = uart::read_byte();
        uart::write_byte(c);
    }
}

