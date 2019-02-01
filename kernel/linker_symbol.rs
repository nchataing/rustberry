#[macro_export]
macro_rules! linker_symbol
{
    { $( static $x:ident ; )* } =>
    {
        extern
        {
            $( static $x: u8; )*
        }
    };

    ( $x:ident ) => (unsafe { &$x as *const u8 as usize });
}
