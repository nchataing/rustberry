#![no_std]
#![feature(asm, lang_items, use_extern_macros)]

#![feature(alloc, allocator_api, global_allocator)]
extern crate alloc;
extern crate rlibc;

extern crate rustberry_io as io;
extern crate rustberry_allocator as allocator;

pub mod fs;
pub mod syscall;
mod application_alloc;

use application_alloc::GlobalAllocator;
#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator;

extern
{
    fn main();
}

#[no_mangle]
pub extern fn start() -> !
{
    unsafe { main(); }
    syscall::exit(0)
}

use core::fmt;
#[lang = "panic_fmt"]
pub extern fn panic_fmt(_msg: fmt::Arguments, _file: &'static str,
                        _line: u32, _column: u32) -> !
{
    // TODO: Show some details
    syscall::exit(101)
}

// This function is needed for linker but should never be called
#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> !
{
    panic!("unimplemented __aeabi_unwind_cpp_pr0");
}
