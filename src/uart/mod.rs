pub mod pl011;
pub mod mini_uart;

#[cfg(not(feature = "mini_uart"))]
pub use self::pl011::*;

#[cfg(feature = "mini_uart")]
pub use self::mini_uart::*;

