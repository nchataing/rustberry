#![no_std]
#![feature(asm, lang_items)]

extern crate rlibc;
mod uart;

#[no_mangle]
pub extern fn kernel_main(_r0: i32, _r1: i32, _atags: i32)
{
    uart::init();
    uart::puts("Hello world !\n");

    loop
    {
        let c = uart::getc();
        uart::putc(c);
    }
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}
