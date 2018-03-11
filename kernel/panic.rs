use mmio;
use gpio;
use drivers::uart::{Uart, Write};
use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments, file: &'static str,
                        line: u32, column: u32) -> !
{
    let _ = write!(Uart,
                   "Kernel panic !\nFile {}, line {}, column {}:\n {}\n",
                   file, line, column, msg);

    gpio::select_pin_function(47, gpio::PinFunction::Output);
    gpio::select_pin_function(35, gpio::PinFunction::Output);
    loop
    {
        gpio::set_pin(47);
        //gpio::clear_pin(35);
        mmio::delay(0x100000);
        gpio::set_pin(47);
        //gpio::clear_pin(35);
        mmio::delay(0x100000);
    }
}
