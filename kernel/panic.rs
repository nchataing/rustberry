use mmio;
use gpio;
use interrupts;
use drivers::uart::{Uart, Write};
use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments, file: &'static str,
                        line: u32, column: u32) -> !
{
    interrupts::disable_all();

    let _ = write!(Uart,
                   "\x1b[31;1mKernel panic !\x1b[0m\n\
                   File {}, line {}, column {}:\n\
                   \x1b[1m{}\x1b[0m\n",
                   file, line, column, msg);

    gpio::select_pin_function(47, gpio::PinFunction::Output);
    gpio::select_pin_function(35, gpio::PinFunction::Output);

    loop
    {
        gpio::set_pin(35);
        gpio::clear_pin(47);
        mmio::delay(100_000_000);
        gpio::clear_pin(35);
        gpio::set_pin(47);
        mmio::delay(100_000_000);
    }
}
