use drivers::mmio;
use mem::*;
use mem::mmu::*;

static mut KERNEL_SECTION_TABLE: KernelSectionTable = KernelSectionTable::new();
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
    pages.register_page(VirtPageId(0), PhysPageId(0), &kernel_text_flags);

    // Kernel stack
    for i in 1 .. fst_text_page
    {
        pages.register_page(VirtPageId(i), PhysPageId(i), &kernel_data_flags);
    }

    // .text
    for i in fst_text_page .. fst_rodata_page
    {
        pages.register_page(VirtPageId(i), PhysPageId(i), &kernel_text_flags);
    }

    // .rodata
    for i in fst_rodata_page .. fst_data_page
    {
        pages.register_page(VirtPageId(i), PhysPageId(i), &kernel_rodata_flags);
    }

    // .data, .bss and after
    for i in fst_data_page .. PAGE_BY_SECTION
    {
        pages.register_page(VirtPageId(i), PhysPageId(i), &kernel_data_flags);
    }

    // Use pages above
    sections.register_page_table(VirtSectionId(0), pages, true);
    sections.register_page_table(VirtSectionId(0x800), pages, true);

    // Standard data sections
    for i in 1 .. mmio::PERIPHERAL_BASE / SECTION_SIZE
    {
        sections.register_section(VirtSectionId(i+0x800), PhysSectionId(i),
                                  &kernel_data_flags, false);
    }

    // Peripheral sections
    let periph_flags = RegionFlags { execute: false, global: true,
        shareable: true, access: RegionAccess::KernelOnly,
        attributes: RegionAttribute::Device };
    for i in mmio::PERIPHERAL_BASE / SECTION_SIZE .. NUM_SECTION_MAX
    {
        sections.register_section(VirtSectionId(i+0x800), PhysSectionId(i),
                                  &periph_flags, false);
    }

    unsafe
    {
        setup_kernel_table(&KERNEL_SECTION_TABLE);
    }
}

pub fn remove_temporary()
{
    unsafe
    {
        KERNEL_SECTION_TABLE.unregister(VirtSectionId(0));
    }
}

const FIRST_HEAP_SECTION : VirtSectionId = VirtSectionId(0xD00);
static mut LAST_HEAP_SECTION : VirtSectionId = FIRST_HEAP_SECTION;

/**
 * Add heap memory for the kernel.
 * Kernel heap memory is mapped between 0xD000_0000 and 0xFFFF_FFFF.
 * It is composed by sections only (allocating page would require kernel heap).
 * This function returns the identifier of the first allocated section.
 * It panics if the requested memory goes above 0xFFFF_FFFF.
 */
pub unsafe fn reserve_kernel_heap_sections(nb: usize) -> VirtSectionId
{
    let first_allocated_section = LAST_HEAP_SECTION;
    for _ in 0 .. nb
    {
        if LAST_HEAP_SECTION.0 >= 0x1000
        {
            panic!("Kernel heap exceeded its maximum size")
        }

        let phys_section = pages::allocate_section();

        let flags = RegionFlags { execute: false, global: true,
            shareable: true, access: RegionAccess::KernelOnly,
            attributes: RegionAttribute::WriteAllocate };

        KERNEL_SECTION_TABLE.register_section(LAST_HEAP_SECTION, phys_section,
                                              &flags, false);

        LAST_HEAP_SECTION.0 += 1;
    }

    mmio::sync_barrier();
    first_allocated_section
}
