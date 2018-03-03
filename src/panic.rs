use mmio;
use gpio;
use core::fmt;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(args: fmt::Arguments, file: &str, line: usize) -> !
{
    unsafe
    {
        let mut ra = mmio::read(gpio::GPFSEL4);
        ra &= !(7<<21);
        ra |= 1<<21;
        mmio::write(gpio::GPFSEL4, ra);

        ra = mmio::read(gpio::GPFSEL3);
        ra &= !(7<<15);
        ra |= 1<<15;
        mmio::write(gpio::GPFSEL3, ra);

        loop
        {
            mmio::write(gpio::GPSET1, 1<<(47-32));
            mmio::write(gpio::GPCLR1, 1<<(35-32));
            mmio::delay(0x100000);
            mmio::write(gpio::GPCLR1, 1<<(47-32));
            mmio::write(gpio::GPSET1, 1<<(35-32));
            mmio::delay(0x100000);
        }
    }
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
