use core::ptr::NonNull;
use memory::*;
use core::alloc::{Alloc, GlobalAlloc, Opaque, Layout};
use allocator::{HeapPageAlloc, Allocator};

const FIRST_HEAP_ADDR: usize = 0x6000_0000;

struct KernelHeapAllocator;
unsafe impl HeapPageAlloc for KernelHeapAllocator
{
    fn first_heap_addr(&self) -> usize { FIRST_HEAP_ADDR }

    unsafe fn reserve_heap_pages(&mut self, nb: usize) -> *mut u8
    {
        kernel_map::reserve_heap_pages(nb).to_addr()
    }

    unsafe fn free_heap_pages(&mut self, nb: usize)
    {
        kernel_map::free_heap_pages(nb)
    }
}

type KernelAllocator = Allocator<KernelHeapAllocator>;

static mut KERNEL_ALLOCATOR: KernelAllocator = Allocator::new(KernelHeapAllocator);


pub struct GlobalKernelAllocator;
unsafe impl GlobalAlloc for GlobalKernelAllocator
{
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque
    {
        KERNEL_ALLOCATOR.alloc(layout).map(|x| x.as_ptr()).unwrap_or(0 as *mut Opaque)
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout)
    {
        match NonNull::new(ptr)
        {
            Some(ptr) => KERNEL_ALLOCATOR.dealloc(ptr, layout),
            None => ()
        }
    }
}

#[lang = "oom"]
#[no_mangle]
pub extern fn rust_oom() -> !
{
    panic!("memory allocation failed");
}

// Purple magic inside
#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> !
{
    panic!("unimplemented __aeabi_unwind_cpp_pr0");
}
