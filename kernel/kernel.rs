#![no_std]
#![feature(asm, panic_info_message, alloc_error_handler)]
#![allow(dead_code)]
#![feature(alloc, allocator_api)]
#[macro_use]
extern crate alloc;
extern crate rlibc;

#[macro_use]
extern crate bitflags;
extern crate goblin;
extern crate plain;

#[macro_use]
extern crate rustberry_drivers as drivers;
extern crate rustberry_allocator as allocator;
extern crate rustberry_io as io;

#[macro_use]
mod log;
#[macro_use]
mod linker_symbol;
mod atag;
pub mod exceptions;
mod filesystem;
pub mod memory;
pub mod panic;
mod process;
mod scheduler;
mod sparse_vec;
pub mod syscall;
mod system_control;
mod timer;

use drivers::*;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::rc::Rc;

use memory::kernel_alloc::GlobalKernelAllocator;

use filesystem::fat32::table::Fat;
use filesystem::Dir;

#[global_allocator]
static ALLOCATOR: GlobalKernelAllocator = GlobalKernelAllocator;

#[no_mangle]
pub extern "C" fn init_memory_map() {
    // This function is called with supervisor stack at 0x8000 and MMU disabled
    memory::physical_alloc::init();
    memory::kernel_map::init();
}

#[no_mangle]
pub extern "C" fn kernel_main() -> () {
    uart::init();
    println!("\x1b[32;1mHello world !\x1b[0m");

    let size = atag::get_mem_size();
    info!("Memory size: {:#x}", size);

    let v1 = vec![1337; 42];
    println!("Dynamic allocation: 1337 = {}", v1[2]);
    drop(v1);

    interrupts::init();
    core_timer::init();

    filesystem::virtualfs::init();

    let mut devfs = filesystem::devfs::DeviceDir::new();
    devfs.add_device("uart".to_owned(), Rc::new(uart::Uart));
    filesystem::virtualfs::get_root().mount(Box::new(devfs), "dev");

    match emmc::init() {
        Ok(sdcard) => {
            let parts;

            match filesystem::mbr_reader::read_partition_table(&sdcard) {
                Ok(partition_table) => parts = partition_table,
                Err(err) => {
                    use filesystem::mbr_reader::Partition;
                    warn!("MBR read failure: {:?}", err);
                    parts = [Partition::new_empty(); 4]
                }
            };

            match Fat::new(Rc::new(sdcard), parts[0].fst_sector as usize, 0) {
                Ok(fs) => {
                    let mut root_dir = fs.root_dir();
                    let root_entries = root_dir.list_entries();
                    for e in &root_entries {
                        e.print()
                    }
                    filesystem::virtualfs::get_root().mount(Box::new(root_dir), "");
                }
                Err(err) => warn!("FAT read failure! {:?}", err),
            }
        }
        Err(err) => warn!("SD card failure: {:?}", err),
    }

    /*unsafe {
        // Each of the following operations must fail !
        mmio::write(0 as *mut u32, 0); // Data abort
        asm!("bx $0" :: "r"(0x2000) :: "volatile"); // Prefetch abort

        let page = memory::kernel_map::reserve_heap_pages(1);
        mmio::write(page.to_addr() as *mut u32, 42);
        memory::kernel_map::free_heap_pages(1);
        println!("{}", mmio::read(page.to_addr() as *mut u32));
    }*/

    unsafe {
        let mut appmap1 = memory::application_map::ApplicationMap::new();
        appmap1.activate();
        let page1 = appmap1.reserve_heap_pages(1).unwrap();
        mmio::write(page1.to_addr() as *mut u32, 42);
        println!(
            "{} @ {:x}",
            mmio::read(page1.to_addr() as *mut u32),
            page1.to_addr()
        );
        mmio::write(0xffff_fffc as *mut u32, 42);

        mmio::instr_barrier();

        let mut appmap2 = memory::application_map::ApplicationMap::new();
        let page2 = appmap2.reserve_heap_pages(1).unwrap();
        appmap2.activate();
        println!(
            "{} @ {:x}",
            mmio::read(page2.to_addr() as *mut u32),
            page2.to_addr()
        );
        mmio::write(page2.to_addr() as *mut u32, 54);
        println!(
            "{} @ {:x}",
            mmio::read(page2.to_addr() as *mut u32),
            page2.to_addr()
        );

        // Should fail (read before write in stack)
        //println!("{} @ 0xffff_fffc", mmio::read(0xffff_fffc as *mut u32));
    }

    println!("π = {}", core::f32::consts::PI);

    random::init();
    match random::generate() {
        Some(rand) => println!("Random -> {:#08x}", rand),
        None => warn!("Random engine timeout"),
    }

    scheduler::init();
    match process::Process::new(
        "init".to_owned(),
        include_bytes!("../target/pi2/release/prgm/init"),
    ) {
        Ok(process) => {
            scheduler::add_process(Box::new(process));
        }
        Err(err) => {
            error!("Couldn't launch init process: {:?}", err);
        }
    }

    match process::Process::new(
        "hello_world".to_owned(),
        include_bytes!("../target/pi2/release/prgm/hello_world"),
    ) {
        Ok(process) => {
            scheduler::add_process(Box::new(process));
        }
        Err(err) => {
            error!("Couldn't launch hello_world process: {:?}", err);
        }
    }

    scheduler::start();
}
