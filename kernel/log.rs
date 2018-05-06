#[macro_export]
macro_rules! warn
{
    ($($arg:tt)*) =>
    {
        println!("\x1b[33;1mWarning:\x1b[0m {}", format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! info
{
    ($($arg:tt)*) =>
    {
        println!("\x1b[34;1mInfo:\x1b[0m {}", format_args!($($arg)*));
    }
}
