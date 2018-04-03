#![no_std]
#![feature(asm, lang_items)]
#![allow(dead_code)]

#![feature(compiler_builtins_lib)]
extern crate compiler_builtins;
extern crate rlibc;

extern crate rustberry_drivers as drivers;
use drivers::uart;
use drivers::mmio;
use core::ptr;

extern "C"
{
    fn reset() -> !;
}

#[no_mangle]
pub extern fn bootloader_main() -> !
{
    uart::init();
    uart::write_str("Rustberry UART bootloader\n");
    uart::write_str("\x03\x03\x03");

    // Wait for kernel size
    let mut size = 0u32;
    for i in 0..4
    {
        let byte = uart::read_byte();
        size |= (byte as u32) << 8*i;
    }

    if size > 0 && size <= 0x10_0000
    {
        uart::write_str("OK");
    }
    else
    {
        uart::write_str("IS");
    }

    for addr in 0..size
    {
        if uart::got_overrun()
        {
            uart::write_str("Bootloader error: UART Overrun\n");
            loop {}
        }
        let data = uart::read_byte();

        if addr >= 0x0100 && addr < 0x8000
        {
            if data != 0
            {
                uart::write_str("Bootloader error: \
                    Cannot write between 0x0100 and 0x8000\n");
                uart::write_str("All these bytes must be 0 in input file\n");

                loop {}
            }
            continue
        }

        let addr_ptr = addr as *mut u8;
        unsafe
        {
            ptr::write_volatile(addr_ptr, data);
        }
    }

    unsafe
    {
        let other_cores_reset_addr = 0x4000 as *mut u32;
        ptr::write_volatile(other_cores_reset_addr, 0);

        mmio::sync_barrier();
        mmio::set_event();

        reset();
    }
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> ! { loop {} }
