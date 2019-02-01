/**
 * This is the interface of an ARM coprocessor 32 bit register.
 * This trait can be automatically derived by the coproc_reg! macro.
 */
pub trait CoprocRegister {
    unsafe fn read() -> u32;
    unsafe fn write(val: u32);

    unsafe fn set_bits(bitmask: u32) {
        let val = Self::read() | bitmask;
        Self::write(val);
    }

    unsafe fn reset_bits(bitmask: u32) {
        let val = Self::read() & !bitmask;
        Self::write(val);
    }
}

/**
 * This is the interface of an ARM coprocessor 64 bit register.
 * This trait can be automatically derived by the coproc_reg64! macro.
 */
pub trait CoprocRegister64 {
    unsafe fn read() -> u64;
    unsafe fn write(val: u64);
}

/**
 * This macro generates a CoprocRegister implementation for a list of
 * coprocessor registers.
 *
 * The read and write functions are implemented with the corresponding MRC and
 * MCR assembly instructions.
 *
 * Usage example:
 * ```
 * coproc_reg! {
 *     REG_NAME : p15, c0, 0, c0, 0;
 * }
 *
 * fn a() {
 *     REG_NAME::write(42);
 * }
 * ```
 */
#[macro_export]
macro_rules! coproc_reg
{
    { $( $reg_id: ident : $coproc: ident, $crn: ident, $opc1: expr,
                          $crm: ident, $opc2: expr; )* } =>
    {
        use crate::coproc_reg::CoprocRegister;
        $( #[allow(non_camel_case_types)] struct $reg_id;
        impl CoprocRegister for $reg_id {
            unsafe fn read() -> u32 {
                let result : u32;
                asm!(concat!("mrc ", stringify!($coproc), ", ",
                             $opc1, ", $0, ", stringify!($crn),
                             ", ", stringify!($crm), ", ", $opc2)
                     : "=r"(result));
                result
            }

            unsafe fn write(val: u32) {
                asm!(concat!("mcr ", stringify!($coproc), ", ",
                             $opc1, ", $0, ", stringify!($crn),
                             ", ", stringify!($crm), ", ", $opc2)
                     :: "r"(val) :: "volatile");
            }
        } )*
    }
}

/// Same as `coproc_reg!` but for 64 bit registers.
#[macro_export]
macro_rules! coproc_reg64
{
    { $( $reg_id: ident : $coproc: ident, $crn: ident, $opc1: expr; )* } =>
    {
        use crate::coproc_reg::CoprocRegister64;
        $( #[allow(non_camel_case_types)] struct $reg_id;
        impl CoprocRegister64 for $reg_id {
            unsafe fn read() -> u64 {
                let low : u32;
                let high : u32;
                asm!(concat!("mrrc ", stringify!($coproc), ", ",
                             $opc1, ", $0, $1, ", stringify!($crn))
                     : "=r"(low), "=r"(high));
                (high as u64) << 32 | low as u64
            }

            unsafe fn write(val: u64) {
                asm!(concat!("mcrr ", stringify!($coproc), ", ",
                             $opc1, ", $0, $1, ", stringify!($crn))
                     :: "r"(val as u32), "r"((val >> 32) as u32) :: "volatile");
            }
        } )*
    }
}
