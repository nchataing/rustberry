use mmio;

/// The GPIO registers base address.
pub const GPIO_BASE : usize = (mmio::PERIPHERAL_BASE + 0x00200000);

/// Controls actuation of pull up/down to ALL GPIO pins.
pub const GPPUD : *mut u32  = (GPIO_BASE + 0x94) as *mut u32;

/// Controls actuation of pull up/down for specific GPIO pin.
pub const GPPUDCLK0 : *mut u32 = (GPIO_BASE + 0x98) as *mut u32;

// GPIO Function Select
pub const GPFSEL0 : *mut u32 = (GPIO_BASE + 0x00) as *mut u32;
pub const GPFSEL1 : *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
pub const GPFSEL2 : *mut u32 = (GPIO_BASE + 0x08) as *mut u32;
pub const GPFSEL3 : *mut u32 = (GPIO_BASE + 0x0C) as *mut u32;
pub const GPFSEL4 : *mut u32 = (GPIO_BASE + 0x10) as *mut u32;
pub const GPFSEL5 : *mut u32 = (GPIO_BASE + 0x14) as *mut u32;

pub const GPSET0  : *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
pub const GPSET1  : *mut u32 = (GPIO_BASE + 0x20) as *mut u32;

pub const GPCLR0  : *mut u32 = (GPIO_BASE + 0x28) as *mut u32;
pub const GPCLR1  : *mut u32 = (GPIO_BASE + 0x2C) as *mut u32;
