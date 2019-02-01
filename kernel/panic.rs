use crate::gpio;
use crate::interrupts;
use crate::mmio;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupts::disable_all();

    print!("\x1b[31;1mKernel panic !\x1b[0m\n");

    if let Some(loc) = info.location() {
        print!(
            "File {}, line {}, column {}:\n",
            loc.file(),
            loc.line(),
            loc.column()
        );
    }

    if let Some(msg) = info.message() {
        print!(" \x1b[1m{}\x1b[0m\n", msg);
    }

    gpio::select_pin_function(47, gpio::PinFunction::Output);
    gpio::select_pin_function(35, gpio::PinFunction::Output);

    loop {
        gpio::set_pin(35);
        gpio::clear_pin(47);
        mmio::delay(100_000_000);
        gpio::clear_pin(35);
        gpio::set_pin(47);
        mmio::delay(100_000_000);
    }
}
