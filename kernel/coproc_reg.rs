/**
 * This is the interface of an ARM coprocessor register.
 * This trait can be automatically derived by the coproc_reg! macro.
 */
pub trait CoprocRegister
{
    unsafe fn read() -> u32;
    unsafe fn write(val: u32);

    unsafe fn set_bits(bitmask: u32)
    {
        let val = Self::read() | bitmask;
        Self::write(val);
    }

    unsafe fn reset_bits(bitmask: u32)
    {
        let val = Self::read() & !bitmask;
        Self::write(val);
    }
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
 * coproc_reg!
 * {
 *     REG_NAME : p15, c0, 0, c0, 0;
 * }
 *
 * fn a()
 * {
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
        use coproc_reg::CoprocRegister;
        $( struct $reg_id;
        impl CoprocRegister for $reg_id
        {
            unsafe fn read() -> u32
            {
                let result : u32;
                asm!(concat!("mrc ", stringify!($coproc), ", ",
                             $opc1, ", $0, ", stringify!($crn),
                             ", ", stringify!($crm), ", ", $opc2)
                     : "=r"(result));
                result
            }

            unsafe fn write(val: u32)
            {
                asm!(concat!("mcr ", stringify!($coproc), ", ",
                             $opc1, ", $0, ", stringify!($crn),
                             ", ", stringify!($crm), ", ", $opc2)
                     :: "r"(val) :: "volatile");
            }
        } )*
    }
}

