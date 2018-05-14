use core::ptr::NonNull;
use memory::*;
use core::alloc::{Alloc, GlobalAlloc, Opaque, Layout};
use allocator::{HeapPageAlloc, Allocator};

struct KernelHeapAllocator;
unsafe impl HeapPageAlloc for KernelHeapAllocator
{
    fn first_heap_addr(&self) -> usize
    {
        kernel_map::FIRST_HEAP_PAGE.to_addr()
    }

    unsafe fn reserve_heap_pages(&mut self, nb: usize) -> usize
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
        // Particular cases for exact page or section demands
        if layout.size() == PAGE_SIZE && layout.align() == PAGE_SIZE
        {
            return physical_alloc::allocate_page().to_addr() as *mut Opaque;
        }
        else if layout.size() == 2*PAGE_SIZE && layout.align() == 2*PAGE_SIZE
        {
            return physical_alloc::allocate_double_page().to_addr() as *mut Opaque;
        }
        else if layout.size() == SECTION_SIZE && layout.align() == SECTION_SIZE
        {
            return physical_alloc::allocate_section().to_addr() as *mut Opaque;
        }

        KERNEL_ALLOCATOR.alloc(layout).map(|x| x.as_ptr()).unwrap_or(0 as *mut Opaque)
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout)
    {
        if layout.size() == PAGE_SIZE && layout.align() == PAGE_SIZE
        {
            physical_alloc::deallocate_page(PageId(ptr as usize / PAGE_SIZE));
        }
        else if layout.size() == 2*PAGE_SIZE && layout.align() == 2*PAGE_SIZE
        {
            physical_alloc::deallocate_double_page(PageId(ptr as usize / PAGE_SIZE));
        }
        else if layout.size() == SECTION_SIZE && layout.align() == SECTION_SIZE
        {
            physical_alloc::deallocate_section(SectionId(ptr as usize / SECTION_SIZE));
        }
        else
        {
            match NonNull::new(ptr)
            {
                Some(ptr) => KERNEL_ALLOCATOR.dealloc(ptr, layout),
                None => ()
            }
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
