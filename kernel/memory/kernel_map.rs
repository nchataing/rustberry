use drivers::mmio;
use atag;
use memory::*;
use memory::mmu::*;

static mut KERNEL_SECTION_TABLE: SectionTable = SectionTable::new();
static mut KERNEL_PAGE_TABLE: PageTable = PageTable::new();

linker_symbol!
{
    static __text;
    static __rodata;
    static __data;
}

/**
 * Create the kernel identity mapping.
 * All addresses below 0x4000_0000 are mapped to themselves.
 * They are accessible by kernel only.
 * All other addresses are unavailable.
 * This function also enables caches. As a consequence,
 * looping code is way faster after this function has been called.
 */
pub fn init()
{
    let sections;
    let pages;
    unsafe
    {
        sections = &mut KERNEL_SECTION_TABLE;
        pages = &mut KERNEL_PAGE_TABLE;
    }

    let kernel_text_flags = RegionFlags { execute: true, global: true,
        shareable: false, access: RegionAccess::KernelReadOnly,
        attributes: RegionAttribute::WriteAllocate };

    let kernel_rodata_flags = RegionFlags { execute: false, global: true,
        shareable: false, access: RegionAccess::KernelReadOnly,
        attributes: RegionAttribute::WriteAllocate };

    let kernel_data_flags = RegionFlags { execute: false, global: true,
        shareable: true, access: RegionAccess::KernelOnly,
        attributes: RegionAttribute::WriteAllocate };

    let fst_text_page = linker_symbol!(__text) / PAGE_SIZE;
    let fst_rodata_page = linker_symbol!(__rodata) / PAGE_SIZE;
    let fst_data_page = linker_symbol!(__data) / PAGE_SIZE;

    // .text.start and ATAGS
    pages.register_page(PageId(0), PageId(0), &kernel_text_flags);

    // Kernel stack
    for i in 1 .. fst_text_page
    {
        pages.register_page(PageId(i), PageId(i), &kernel_data_flags);
    }

    // .text
    for i in fst_text_page .. fst_rodata_page
    {
        pages.register_page(PageId(i), PageId(i), &kernel_text_flags);
    }

    // .rodata
    for i in fst_rodata_page .. fst_data_page
    {
        pages.register_page(PageId(i), PageId(i), &kernel_rodata_flags);
    }

    // .data, .bss and after
    for i in fst_data_page .. PAGE_BY_SECTION
    {
        pages.register_page(PageId(i), PageId(i), &kernel_data_flags);
    }

    // Use pages above
    sections.register_page_table(SectionId(0), pages, true);

    // Standard data sections
    let memory_size = atag::get_mem_size();
    let nb_ram_sections = memory_size / SECTION_SIZE;
    for i in 1 .. nb_ram_sections
    {
        sections.register_section(SectionId(i), SectionId(i),
                                  &kernel_data_flags, false);
    }

    // Peripheral sections
    let periph_flags = RegionFlags { execute: false, global: true,
        shareable: true, access: RegionAccess::KernelOnly,
        attributes: RegionAttribute::Device };
    for i in nb_ram_sections .. FIRST_VIRTUAL_SECTION
    {
        sections.register_section(SectionId(i), SectionId(i),
                                  &periph_flags, false);
    }

    unsafe
    {
        setup_kernel_table(&KERNEL_SECTION_TABLE as *const SectionTable);
    }
}

const FIRST_HEAP_PAGE : PageId = PageId(0x600_00);
static mut LAST_HEAP_PAGE : PageId = FIRST_HEAP_PAGE;

/**
 * Add heap memory for the kernel.
 * Kernel heap memory is mapped between 0x6000_0000 and 0x7FFF_FFFF.
 * This function returns the identifier of the first allocated page.
 * It panics if the requested memory goes above 0x7FFF_FFFF.
 */
pub unsafe fn reserve_heap_pages(nb: usize) -> PageId
{
    let first_allocated_page = LAST_HEAP_PAGE;
    for _ in 0 .. nb
    {
        if LAST_HEAP_PAGE.0 >= 0x100000
        {
            panic!("Kernel heap exceeded its maximum size")
        }

        let phys_page = physical_alloc::allocate_page();

        let flags = RegionFlags { execute: false, global: true,
            shareable: true, access: RegionAccess::KernelOnly,
            attributes: RegionAttribute::WriteAllocate };

        KERNEL_SECTION_TABLE.register_page(LAST_HEAP_PAGE, phys_page, &flags);

        LAST_HEAP_PAGE.0 += 1;
    }

    mmio::sync_barrier();

    #[cfg(feature = "trace_kernel_heap_pages")]
    info!("Allocated {} kernel heap pages at {}", nb, first_allocated_page);

    first_allocated_page
}

pub unsafe fn free_heap_pages(nb: usize)
{
    for _ in 0 .. nb
    {
        if LAST_HEAP_PAGE.0 <= FIRST_HEAP_PAGE.0
        {
            panic!("Cannot free empty kernel heap")
        }
        LAST_HEAP_PAGE.0 -= 1;

        let paddr = translate_addr(LAST_HEAP_PAGE.to_addr())
            .expect("Kernel heap page already deallocated");
        KERNEL_SECTION_TABLE.unregister_page(LAST_HEAP_PAGE);
        cache::tlb::invalidate_page(LAST_HEAP_PAGE);
        physical_alloc::deallocate_page(PageId(paddr as usize / PAGE_SIZE));
    }

    #[cfg(feature = "trace_kernel_heap_pages")]
    info!("Deallocated {} kernel heap pages", nb);
}

pub fn translate_addr(vaddr: *mut u8) -> Option<*mut u8>
{
    unsafe
    {
        KERNEL_SECTION_TABLE.translate_addr(vaddr)
    }
}

