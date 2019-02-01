use crate::syscall;
use allocator::{Allocator, HeapPageAlloc};
use core::alloc::{Alloc, GlobalAlloc, Layout};
use core::ptr::NonNull;

struct HeapAllocator;
unsafe impl HeapPageAlloc for HeapAllocator {
    fn first_heap_addr(&self) -> usize {
        0xA000_0000
    }

    unsafe fn reserve_heap_pages(&mut self, nb: usize) -> usize {
        syscall::reserve_heap_pages(nb as isize)
    }

    unsafe fn free_heap_pages(&mut self, nb: usize) {
        syscall::reserve_heap_pages(-(nb as isize));
    }
}

type AppAllocator = Allocator<HeapAllocator>;
static mut ALLOCATOR: AppAllocator = Allocator::new(HeapAllocator);

pub struct GlobalAllocator;
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATOR
            .alloc(layout)
            .map(|x| x.as_ptr())
            .unwrap_or(0 as *mut u8)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match NonNull::new(ptr) {
            Some(ptr) => ALLOCATOR.dealloc(ptr, layout),
            None => (),
        }
    }
}

#[lang = "oom"]
extern "C" fn rust_oom(_: Layout) -> ! {
    panic!("memory allocation failed");
}
