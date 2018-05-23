#![no_std]
#![feature(asm, lang_items, const_fn, iterator_step_by)]
#![allow(dead_code)]

#![feature(alloc, allocator_api, global_allocator)]
#[macro_use] extern crate alloc;
extern crate rlibc;

#[macro_use] extern crate bitflags;
extern crate plain;
extern crate goblin;

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
mod sparse_vec;
mod scheduler;

use drivers::*;

// use alloc::boxed::Box;
// use alloc::borrow::ToOwned;

use memory::kernel_alloc::GlobalKernelAllocator;
#[global_allocator]
static ALLOCATOR: GlobalKernelAllocator = GlobalKernelAllocator;

#[no_mangle]
pub extern fn init_memory_map()
{
    // This function is called with supervisor stack at 0x8000 and MMU disabled
    memory::physical_alloc::init();
    memory::kernel_map::init();
}

#[no_mangle]
pub extern fn kernel_main() -> ()
{
    uart::init();
    println!("\x1b[32;1mHello world !\x1b[0m");

    let size = atag::get_mem_size();
    info!("Memory size: {:#x}", size);

    let v1 = vec![1337;42];
    println!("Dynamic allocation: 1337 = {}", v1[2]);
    drop(v1);

    interrupts::init();
    core_timer::init();

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

            let parts; 

            match filesystem::mbr_reader::read_partition_table(&sdcard)
            {
                Ok(partition_table) => parts = partition_table,
                Err(err) => {
                    use filesystem::mbr_reader::Partition;
                    warn!("MBR read failure: {:?}", err);
                    parts = [Partition::new_empty(); 4]
                }
            };

            match filesystem::fs::read_bpb_info(sdcard, 
                                             parts[0].fst_sector as usize)
            {
                Ok(bpb) => println!("{:?}", bpb),
                Err(err) => warn!("FAT read failure! {:?}", err)
            }
        },
        Err(err) => warn!("SD card failure: {:?}", err)
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
        let mut appmap1 = memory::application_map::ApplicationMap::new();
        appmap1.activate();
        let page1 = appmap1.reserve_heap_pages(1).unwrap();
        mmio::write(page1.to_addr() as *mut u32, 42);
        println!("{} @ {:x}", mmio::read(page1.to_addr() as *mut u32), page1.to_addr());
        mmio::write(0xffff_fffc as *mut u32, 42);

        mmio::instr_barrier();

        let mut appmap2 = memory::application_map::ApplicationMap::new();
        let page2 = appmap2.reserve_heap_pages(1).unwrap();
        appmap2.activate();
        println!("{} @ {:x}", mmio::read(page2.to_addr() as *mut u32), page2.to_addr());
        mmio::write(page2.to_addr() as *mut u32, 54);
        println!("{} @ {:x}", mmio::read(page2.to_addr() as *mut u32), page2.to_addr());

        // Should fail (read before write in stack)
        //println!("{} @ 0xffff_fffc", mmio::read(0xffff_fffc as *mut u32));
    }

    println!("π = {}", core::f32::consts::PI);

    random::init();
    match random::generate()
    {
        Some(rand) => println!("Random -> {:#08x}", rand),
        None => warn!("Random engine timeout")
    }
/*
    scheduler::init();
    match process::Process::new("init".to_owned(),
        include_bytes!("../target/pi2/release/prgm/syscall_loop"))
    {
        Ok(process) =>
        {
            scheduler::add_process(Box::new(process));
        },
        Err(err) =>
        {
            error!("Couldn't launch init process: {:?}", err);
        },
    }

    scheduler::start();
*/
}
