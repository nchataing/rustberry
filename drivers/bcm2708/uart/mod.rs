pub mod mini_uart;
pub mod pl011;

#[cfg(not(feature = "mini_uart"))]
pub use self::pl011::*;

#[cfg(feature = "mini_uart")]
pub use self::mini_uart::*;
