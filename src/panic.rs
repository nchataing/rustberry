use mmio;
use gpio;
use uart::{Uart, Write};
use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(args: fmt::Arguments, file: &str, line: usize) -> !
{
    let _ = write!(Uart, "Kernel panic!\n{} l{}: {}", file, line, args);

    gpio::select_pin_function(47, gpio::PinFunction::Output);
    gpio::select_pin_function(35, gpio::PinFunction::Output);

    loop
    {
        gpio::set_pin(47);
        gpio::clear_pin(35);
        mmio::delay(0x100000);
        gpio::set_pin(47);
        gpio::clear_pin(35);
        mmio::delay(0x100000);
    }
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
