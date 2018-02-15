#![no_std]
#![feature(asm, lang_items)]

// Memory-Mapped I/O output
unsafe fn mmio_write(addr: i32, data: i32)
{
	*(addr as *mut i32) = data;
}
 
// Memory-Mapped I/O input
unsafe fn mmio_read(addr: i32) -> i32
{
	*(addr as *const i32)
}
 
// Loop <delay> times in a way that the compiler won't optimize away
unsafe fn delay(count: i32)
{
	asm!("__delay_42:
            subs $0, $0, #1
            bne __delay_42
            "
		 : 
         : "r"(count)
         : 
         : "volatile"
    );
}


// The GPIO registers base address.
const GPIO_BASE : i32 = 0x3F200000; // for raspi2 & 3, 0x20200000 for raspi1

// The offsets for reach register.

// Controls actuation of pull up/down to ALL GPIO pins.
const GPPUD : i32 = (GPIO_BASE + 0x94);

// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0 : i32 = (GPIO_BASE + 0x98);

// The base address for UART.
const UART0_BASE : i32 = 0x3F201000; // for raspi2 & 3, 0x20201000 for raspi1

// The offsets for reach register for the UART.
const UART0_DR     : i32 = (UART0_BASE + 0x00);
const UART0_RSRECR : i32 = (UART0_BASE + 0x04);
const UART0_FR     : i32 = (UART0_BASE + 0x18);
const UART0_ILPR   : i32 = (UART0_BASE + 0x20);
const UART0_IBRD   : i32 = (UART0_BASE + 0x24);
const UART0_FBRD   : i32 = (UART0_BASE + 0x28);
const UART0_LCRH   : i32 = (UART0_BASE + 0x2C);
const UART0_CR     : i32 = (UART0_BASE + 0x30);
const UART0_IFLS   : i32 = (UART0_BASE + 0x34);
const UART0_IMSC   : i32 = (UART0_BASE + 0x38);
const UART0_RIS    : i32 = (UART0_BASE + 0x3C);
const UART0_MIS    : i32 = (UART0_BASE + 0x40);
const UART0_ICR    : i32 = (UART0_BASE + 0x44);
const UART0_DMACR  : i32 = (UART0_BASE + 0x48);
const UART0_ITCR   : i32 = (UART0_BASE + 0x80);
const UART0_ITIP   : i32 = (UART0_BASE + 0x84);
const UART0_ITOP   : i32 = (UART0_BASE + 0x88);
const UART0_TDR    : i32 = (UART0_BASE + 0x8C);

fn uart_init()
{
	unsafe 
    {
        // Disable UART0.
        mmio_write(UART0_CR, 0x00000000);
        // Setup the GPIO pin 14 && 15.
    
        // Disable pull up/down for all GPIO pins & delay for 150 cycles.
        mmio_write(GPPUD, 0x00000000);
        delay(150);
    
        // Disable pull up/down for pin 14,15 & delay for 150 cycles.
        mmio_write(GPPUDCLK0, (1 << 14) | (1 << 15));
        delay(150);
    
        // Write 0 to GPPUDCLK0 to make it take effect.
        mmio_write(GPPUDCLK0, 0x00000000);
    
        // Clear pending interrupts.
        mmio_write(UART0_ICR, 0x7FF);
    
        // Set integer & fractional part of baud rate.
        // Divider = UART_CLOCK/(16 * Baud)
        // Fraction part register = (Fractional part * 64) + 0.5
        // UART_CLOCK = 3000000; Baud = 115200.
    
        // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
        mmio_write(UART0_IBRD, 1);
        // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
        mmio_write(UART0_FBRD, 40);
    
        // Enable FIFO & 8 bit data transmissio (1 stop bit, no parity).
        mmio_write(UART0_LCRH, (1 << 4) | (1 << 5) | (1 << 6));
    
        // Mask all interrupts.
        mmio_write(UART0_IMSC, (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) |
                            (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10));
    
        // Enable UART0, receive & transfer part of UART.
        mmio_write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
    }
}

fn uart_putc(c: u8)
{
	unsafe 
    {
        // Wait for UART to become ready to transmit.
	    while (mmio_read(UART0_FR) & (1 << 5)) != 0 { }
	    mmio_write(UART0_DR, c as i32);
    }
}
 
fn uart_getc() -> u8
{
    unsafe
    {
        // Wait for UART to have received something.
        while (mmio_read(UART0_FR) & (1 << 4)) != 0 { }
        mmio_read(UART0_DR) as u8
    }
}

#[no_mangle]
pub extern fn kernel_main(r0: i32, r1: i32, atags: i32)
{
    uart_init();
    uart_putc(72);
    uart_putc(101);
    uart_putc(108);
    uart_putc(108);
    uart_putc(44);
    uart_putc(92);
    uart_putc(107);
    uart_putc(101);
    uart_putc(114);
    uart_putc(110);
    uart_putc(101);
    uart_putc(111);
    uart_putc(92);
    uart_putc(87);
    uart_putc(111);
    uart_putc(114);
    uart_putc(108);
    uart_putc(100);
    uart_putc(33);
 
	loop {
		uart_putc(uart_getc());
    }
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}
