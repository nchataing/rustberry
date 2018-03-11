use mmio;

/// The GPIO registers base address.
pub const GPIO_BASE : usize = (mmio::PERIPHERAL_BASE + 0x00200000);

pub enum PinFunction
{
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

pub fn select_pin_function(pin_id: u8, function: PinFunction)
{
    assert!(pin_id < 54);

    let pin_reg = pin_id / 10;
    let pin_reg_pos = 3 * (pin_id % 10);

    let gpfsel_addr = (GPIO_BASE + (4 * pin_reg) as usize) as *mut u32;
    unsafe
    {
        let mut fsel = mmio::read(gpfsel_addr);
        fsel &= !(0b111 << pin_reg_pos);
        fsel |= (function as u32) << pin_reg_pos;
        mmio::write(gpfsel_addr, fsel);
    }
}


/// GPIO Pin Output Set
const GPSET0 : *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPSET1 : *mut u32 = (GPIO_BASE + 0x20) as *mut u32;

pub fn set_pin(pin_id: u8)
{
    assert!(pin_id < 54);
    if pin_id < 32
    {
        unsafe
        {
            mmio::write(GPSET0, 1<<pin_id);
        }
    }
    else
    {
        unsafe
        {
            mmio::write(GPSET1, 1<<(pin_id-32));
        }
    }
}


/// GPIO Pin Output Clear
const GPCLR0  : *mut u32 = (GPIO_BASE + 0x28) as *mut u32;
const GPCLR1  : *mut u32 = (GPIO_BASE + 0x2C) as *mut u32;

pub fn clear_pin(pin_id: u8)
{
    assert!(pin_id < 54);
    if pin_id < 32
    {
        unsafe
        {
            mmio::write(GPCLR0, 1<<pin_id);
        }
    }
    else
    {
        unsafe
        {
            mmio::write(GPCLR1, 1<<(pin_id-32));
        }
    }
}


/// GPIO Pin Level
const GPLEV0 : *const u32 = (GPIO_BASE + 0x34) as *const u32;
const GPLEV1 : *const u32 = (GPIO_BASE + 0x38) as *const u32;

/// Return true if the requested pin is high
pub fn read_pin(pin_id: u8) -> bool
{
    assert!(pin_id < 54);
    if pin_id < 32
    {
        unsafe
        {
            mmio::read(GPLEV0) & (1<<pin_id) != 0
        }
    }
    else
    {
        unsafe
        {
            mmio::read(GPLEV1) & (1<<(pin_id-32)) != 0
        }
    }
}


/// Controls actuation of pull up/down to ALL GPIO pins.
const GPPUD : *mut u32  = (GPIO_BASE + 0x94) as *mut u32;

/// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0 : *mut u32 = (GPIO_BASE + 0x98) as *mut u32;
const GPPUDCLK1 : *mut u32 = (GPIO_BASE + 0x9C) as *mut u32;

pub enum PullMode
{
    Disabled = 0b00,
    PullDown = 0b01,
    PullUp = 0b10,
}

/// Set the pull up/down mode for the given pin.
pub fn set_pull_mode(pin_id: u8, mode: PullMode)
{
    assert!(pin_id < 54);
    unsafe
    {
        mmio::write(GPPUD, mode as u32);
        mmio::delay(150);

        if pin_id < 32
        {
            mmio::write(GPPUDCLK0, 1<<pin_id);
        }
        else
        {
            mmio::write(GPPUDCLK1, 1<<(pin_id-32));
        }
        mmio::delay(150);

        mmio::write(GPPUD, 0);
        mmio::write(GPPUDCLK0, 0);
        mmio::write(GPPUDCLK1, 0);
    }
}
