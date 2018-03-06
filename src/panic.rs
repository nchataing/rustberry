use mmio;
use gpio;
use uart;
use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(msg: fmt::Arguments, file: &'static str,
                               line: u32, column: u32) -> !
{
    uart::write_str("Kernel panic !");

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

