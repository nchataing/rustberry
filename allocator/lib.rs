#![no_std]
#![feature(const_fn, allocator_api)]

use core::ptr::NonNull;
use core::alloc::{Alloc, Layout, Opaque, AllocErr};

const PAGE_SIZE : usize = 0x1000;

/**
 * This trait defines a heap page allocator that can be used as a base for the
 * memory allocator.
 */
pub unsafe trait HeapPageAlloc
{
    /// This function must return the first heap address allocated.
    fn first_heap_addr(&self) -> usize;

    /**
     * This function reserves new heap pages contiguous to the already
     * allocated ones.
     * It returns the first newly allocated address.
     */
    unsafe fn reserve_heap_pages(&mut self, nb: usize) -> *mut u8;

    /// This function deallocate `nb` pages at the end of the heap.
    unsafe fn free_heap_pages(&mut self, nb: usize);
}

#[repr(C)]
struct BlockDescriptor(usize);

impl BlockDescriptor
{
    fn set_size(&mut self, size: usize)
    {
        self.0 = (size << 2) | (self.0 & 0b01);
    }

    fn get_size(&self) -> usize
    {
        self.0 >> 2
    }

    fn set_free(&mut self) { self.0 |= 1; }
    fn set_full(&mut self) { self.0 &= !1; }
    fn is_free(&self) -> bool { self.0 & 1 != 0 }

    unsafe fn to_footer(&self) -> *mut BlockDescriptor
    {
        (self as *const BlockDescriptor).offset(self.get_size() as isize + 1)
            as usize as *mut BlockDescriptor
    }
    unsafe fn to_header(&self) -> *mut BlockDescriptor
    {
        (self as *const BlockDescriptor).offset(-(self.get_size() as isize) - 1)
            as usize as *mut BlockDescriptor
    }

}

#[repr(C)]
struct FreeBlock
{
    descr: BlockDescriptor,
    next: Option<NonNull<FreeBlock>>,
    prev: Option<NonNull<FreeBlock>>,
}

pub struct Allocator<PageAllocator>
{
    page_allocator: PageAllocator,
    first_free_block: Option<NonNull<FreeBlock>>
}

impl<PageAllocator: HeapPageAlloc> Allocator<PageAllocator>
{
    pub const fn new(page_allocator: PageAllocator) -> Self
    {
        Allocator { page_allocator, first_free_block: None }
    }

    /**
     * Add `nb` virtual pages to the kernel stack and initialize them for
     * usage with this allocator.
     */
    unsafe fn add_pages(&mut self, nb: usize)
    {
        let fst_new_page = self.page_allocator.reserve_heap_pages(nb);
        let header;
        if fst_new_page as usize == self.page_allocator.first_heap_addr()
        {
            let fst_header = fst_new_page as *mut BlockDescriptor;
            let fst_footer = fst_header.offset(1);

            (*fst_header).set_size(0);
            (*fst_header).set_full();
            (*fst_footer).set_size(0);
            (*fst_footer).set_full();

            header = fst_footer.offset(1) as *mut FreeBlock;
        }
        else { header = fst_new_page as *mut FreeBlock }

        let footer = (fst_new_page as *mut BlockDescriptor)
            .offset((nb * PAGE_SIZE/4) as isize - 1);

        let size = ((footer as usize - header as usize) / 4) - 1;
        (*header).descr.set_size(size);
        (*header).descr.set_free();

        (*footer).set_size(size);
        (*footer).set_free();

        (*header).next = self.first_free_block;
        (*header).prev = None;
        self.first_free_block = Some(NonNull::new_unchecked(header));

        self.coalesce_before(header);
    }

    /**
     * Remove a free block from the linked list
     */
    unsafe fn link_through(&mut self, free_block: *mut FreeBlock)
    {
        match (*free_block).prev
        {
            None => self.first_free_block = (*free_block).next,
            Some(ref mut prev_free_block) =>
                prev_free_block.as_mut().next = (*free_block).next
        }
        match (*free_block).next
        {
            None => (),
            Some(ref mut next_free_block) =>
                next_free_block.as_mut().prev = (*free_block).prev
        }
    }

    unsafe fn link_first(&mut self, free_block: *mut FreeBlock)
    {
        (*free_block).next = self.first_free_block;
        (*free_block).prev = None;
        if let Some(ref mut old_first) = self.first_free_block
        {
            old_first.as_mut().prev = Some(NonNull::new_unchecked(free_block));
        }
        self.first_free_block = Some(NonNull::new_unchecked(free_block));
    }

    /**
     * Merge the free block at `block_addr` with the previous block if it is
     * free. In that case it removes the current block from the linked list.
     */
    unsafe fn coalesce_before(&mut self, block_addr: *mut FreeBlock)
    {
        let header_addr = block_addr as *mut BlockDescriptor;

        let prev_footer = header_addr.offset(-1);
        if (*prev_footer).is_free()
        {
            let prev_header = (*prev_footer).to_header();

            let cur_size = (*header_addr).get_size();
            let new_size = (*prev_header).get_size() + cur_size  + 2;
            (*prev_header).set_size(new_size);

            let footer = (*header_addr).to_footer();
            (*footer).set_size(new_size);

            self.link_through(block_addr);
        }
    }

    /**
     * Merge the free block at `block_addr` with the next block if it is free.
     * In that case, it removes the next block from the linked list.
     */
    unsafe fn coalesce_after(&mut self, block_addr: *mut FreeBlock)
    {
        let descr_addr = block_addr as *mut BlockDescriptor;
        let next_header = (*descr_addr).to_footer().offset(1);
        if (*next_header).is_free()
        {
            self.coalesce_before(next_header as *mut FreeBlock)
        }
    }

    /**
     * Merge the free block with adjacent blocks. Only the leftmost new block
     * is included in the linked list.
     */
    unsafe fn coalesce(&mut self, block_addr: *mut FreeBlock)
    {
        self.coalesce_after(block_addr);
        self.coalesce_before(block_addr);
    }

    /**
     * Cut the block at `block_addr` to give it the size `new_size`.
     * `new_size` is the future block size of the left part given in block
     * The left part keeps its status (free/full and linked list position).
     * The right part is considered as a new free block and gets inserted in
     * the linked list.
     */
    unsafe fn split(&mut self, block_addr: *mut BlockDescriptor, new_size: usize)
    {
        let cur_size = (*block_addr).get_size();
        if cur_size - new_size < 4 { return; }

        let new_footer = block_addr.offset(new_size as isize + 1);
        let new_header = new_footer.offset(1);
        let footer_addr = (*block_addr).to_footer();

        let remaining_size = cur_size - new_size - 2;

        (*block_addr).set_size(new_size);
        if (*block_addr).is_free()
        {
            (*new_footer).set_free();
        }
        else
        {
            (*new_footer).set_full();
        }
        (*new_footer).set_size(new_size);
        (*new_header).set_size(remaining_size);
        (*new_header).set_free();
        (*footer_addr).set_size(remaining_size);
        (*footer_addr).set_free();

        let new_free_block = new_header as *mut FreeBlock;
        self.link_first(new_free_block);
    }

    /**
     * Moves the limit between the current block at `header_addr` and the
     * previous block `padding_size` word forward, and then tries to split the
     * previous block to create a new free block.
     * It returns the pointer to the new block header.
     * The current block must have been deleted from the linked list, and the
     * previous block must be full.
     */
    unsafe fn eliminate_padding(&mut self, header_addr: *mut BlockDescriptor, padding_size: usize)
        -> *mut BlockDescriptor
    {
        if padding_size == 0 { return header_addr; }

        let prev_footer = header_addr.offset(-1);
        let prev_size = (*prev_footer).get_size();
        let prev_header = (*prev_footer).to_header();

        let new_header = header_addr.offset(padding_size as isize);
        let new_footer = new_header.offset(-1);

        let size = (*header_addr).get_size();
        let cur_footer = (*header_addr).to_footer();

        (*prev_header).set_size(prev_size + padding_size);
        (*cur_footer).set_size(size - padding_size);
        (*new_footer).set_size(prev_size + padding_size);
        (*new_footer).set_full();
        (*new_header).set_size(size - padding_size);
        (*new_header).set_free();

        self.split(prev_header, prev_size);

        new_header
    }
}

/**
 * Align the `base_addr` adress to respect the layout.
 */
fn align_addr(base_addr: usize, layout: Layout) -> usize
{
    if base_addr % layout.align() == 0 { base_addr }
    else
    {
        ((base_addr + layout.align()) & !(layout.align() - 1))
    }
}

unsafe impl<PageAllocator: HeapPageAlloc> Alloc for Allocator<PageAllocator>
{
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr>
    {
        let mut cur_free_block = self.first_free_block;
        while let Some(cur_block) = cur_free_block
        {
            let cur_block = cur_block.as_ptr();
            let cur_header = cur_block as *mut BlockDescriptor;
            let aligned_addr = align_addr(cur_header.offset(1) as usize, layout);
            let padding_size = (aligned_addr as usize - cur_header as usize + 3) / 4 - 1;
            // Size in word which is to be allocated
            let size = (layout.size() + 3) / 4;

            if size + padding_size <= (*cur_header).get_size()
            {
                self.link_through(cur_header as *mut FreeBlock);
                let cur_header = self.eliminate_padding(cur_header, padding_size);
                self.split(cur_header, size);
                (*cur_header).set_full();
                (*cur_header).set_size(size);
                let footer = (*cur_header).to_footer();
                (*footer).set_full();
                (*footer).set_size(size);

                return Ok(NonNull::new_unchecked(aligned_addr as *mut Opaque));
            }
            else
            {
                cur_free_block = (*cur_block).next
            }
        }

        self.add_pages(((layout.align() + layout.size()) / PAGE_SIZE) + 1);
        self.alloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<Opaque>, _: Layout)
    {
        let header_addr = (ptr.as_ptr() as *mut u32).offset(-1) as *mut FreeBlock;
        (*header_addr).descr.set_free();

        let footer_addr = (*header_addr).descr.to_footer();
        (*footer_addr).set_free();

        self.link_first(header_addr);
        self.coalesce(header_addr);
    }
}
