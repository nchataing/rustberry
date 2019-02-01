#![no_std]
#![feature(asm, lang_items, panic_info_message)]
#![feature(alloc, allocator_api)]
extern crate alloc;
extern crate rlibc;

extern crate rustberry_allocator as allocator;
pub extern crate rustberry_io as io;

mod application_alloc;
pub mod fs;
pub mod syscall;

use application_alloc::GlobalAllocator;
#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator;

pub static mut STDIO: Option<fs::File> = None;

#[macro_export]
macro_rules! print
{
    ($($arg:tt)*) =>
    {{
        use $crate::io::Write;
        unsafe
        {
            let _ = write!($crate::STDIO.as_mut().unwrap(), $($arg)*);
        }
    }}
}

#[macro_export]
macro_rules! println
{
    ($($arg:tt)*) =>
    {{
        use $crate::io::Write;
        unsafe
        {
            let _ = writeln!($crate::STDIO.as_mut().unwrap(), $($arg)*);
        }
    }}
}

extern "C" {
    fn main();
}

#[no_mangle]
pub extern "C" fn start() -> ! {
    unsafe {
        STDIO = Some(fs::File::open("dev/uart").unwrap());
        main();
    }
    syscall::exit(0)
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        print!(
            "\x1b[31;1mKernel panic !\x1b[0m\n\
             File {}, line {}, column {}:\n\
             \x1b[1m{}\x1b[0m\n",
            loc.file(),
            loc.line(),
            loc.column(),
            info.message().unwrap()
        );
    } else {
        print!(
            "\x1b[31;1mKernel panic !\x1b[0m\n\
             \x1b[1m{}\x1b[0m\n",
            info.message().unwrap()
        );
    }
    syscall::exit(101)
}

// This function is needed for linker but should never be called
#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> ! {
    panic!("unimplemented __aeabi_unwind_cpp_pr0");
}
