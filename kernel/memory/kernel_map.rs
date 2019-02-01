/*!
 * The kernel memory map is organized as follows:
 * 0x0000_0000 - 0x400F_FFFF: Identity mapping, access to the corresponding physical addresses.
 *   0x00000 - 0x00FFF: First page, interrupt vectors and ATAGS
 *   0x01000 - 0x01FFF: Abort heap, used for reserving more memory on aborts
 *   0x02000 - 0x07FFF: RESERVED (physical adresses of first supervisor stack pages)
 *   0x08000 - 0xFFFFF: Kernel code and static variables
 *   ...: dynamic allocation physical space
 *   0x3F00_0000 - 0x3FFF_FFFF: BCM 2708 peripheral MMIO
 *   0x4000_0000 - 0x400F_FFFF: Quad-A7 peripheral MMIO
 * 0x5000_0000 - 0x6FFF_FFFF: Kernel heap, growing up
 * 0x7000_0000 - 0x7FFF_FFFF: Supervisor (main kernel mode) stack, growing down
 */

use atag;
use drivers::mmio;
use memory::mmu::*;
use memory::*;

static mut KERNEL_SECTION_TABLE: SectionTable = SectionTable::new();
static mut KERNEL_PAGE_TABLE: PageTable = PageTable::new();

linker_symbol! {
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
pub fn init() {
    let sections;
    let pages;
    unsafe {
        sections = &mut KERNEL_SECTION_TABLE;
        pages = &mut KERNEL_PAGE_TABLE;
    }

    let kernel_text_flags = RegionFlags {
        execute: true,
        global: true,
        shareable: false,
        access: RegionAccess::KernelReadOnly,
        attributes: RegionAttribute::WriteAllocate,
    };

    let kernel_rodata_flags = RegionFlags {
        execute: false,
        global: true,
        shareable: false,
        access: RegionAccess::KernelReadOnly,
        attributes: RegionAttribute::WriteAllocate,
    };

    let kernel_data_flags = RegionFlags {
        execute: false,
        global: true,
        shareable: true,
        access: RegionAccess::KernelOnly,
        attributes: RegionAttribute::WriteAllocate,
    };

    let fst_text_page = linker_symbol!(__text) / PAGE_SIZE;
    let fst_rodata_page = linker_symbol!(__rodata) / PAGE_SIZE;
    let fst_data_page = linker_symbol!(__data) / PAGE_SIZE;

    // .text.start and ATAGS
    pages.register_page(PageId(0), PageId(0), &kernel_text_flags);

    // Kernel initial stack & abort stack
    for i in 1..fst_text_page {
        pages.register_page(PageId(i), PageId(i), &kernel_data_flags);
    }

    // .text
    for i in fst_text_page..fst_rodata_page {
        pages.register_page(PageId(i), PageId(i), &kernel_text_flags);
    }

    // .rodata
    for i in fst_rodata_page..fst_data_page {
        pages.register_page(PageId(i), PageId(i), &kernel_rodata_flags);
    }

    // .data, .bss and after
    for i in fst_data_page..PAGE_BY_SECTION {
        pages.register_page(PageId(i), PageId(i), &kernel_data_flags);
    }

    unsafe {
        // Use pages above / safe as KERNEL_SECTION_TABLE is never destroyed
        sections.register_page_table(SectionId(0), pages, true);
    }

    // Standard data sections
    let memory_size = atag::get_mem_size();
    let nb_ram_sections = memory_size / SECTION_SIZE;
    for i in 1..nb_ram_sections {
        sections.register_section(SectionId(i), SectionId(i), &kernel_data_flags, false);
    }

    // Peripheral sections
    let periph_flags = RegionFlags {
        execute: false,
        global: true,
        shareable: true,
        access: RegionAccess::KernelOnly,
        attributes: RegionAttribute::Device,
    };
    for i in nb_ram_sections..FIRST_VIRTUAL_SECTION {
        sections.register_section(SectionId(i), SectionId(i), &periph_flags, false);
    }

    // Register first supervisor stack pages
    for i in 2..8 {
        sections.register_page(PageId(0x7FF_F8 + i), PageId(i), &kernel_data_flags);
    }

    unsafe {
        setup_kernel_table(&KERNEL_SECTION_TABLE as *const SectionTable);
    }
}

pub const FIRST_HEAP_PAGE: PageId = PageId(0x500_00);
static mut LAST_HEAP_PAGE: PageId = FIRST_HEAP_PAGE;
pub const STACK_PAGE_LIMIT: PageId = PageId(0x700_00);
static mut LAST_STACK_PAGE: PageId = PageId(0x7FF_FA);
pub const FIRST_APPLICATION_PAGE: PageId = PageId(0x800_00);

/**
 * Add supervisor stack memory.
 * Supervisor stack is mapped between 0x7000_0000 and 0x7FFF_FFFF.
 * It panics if the requested page goes below 0x7000_0000.
 */
pub fn add_svc_stack_pages(nb: usize) {
    unsafe {
        for _ in 0..nb {
            if LAST_STACK_PAGE.0 <= STACK_PAGE_LIMIT.0 {
                panic!("Supervisor stack exceeded its maximum size")
            }
            LAST_STACK_PAGE.0 -= 1;

            let phys_page = physical_alloc::allocate_page();

            let flags = RegionFlags {
                execute: false,
                global: true,
                shareable: true,
                access: RegionAccess::KernelOnly,
                attributes: RegionAttribute::WriteAllocate,
            };

            KERNEL_SECTION_TABLE.register_page(LAST_STACK_PAGE, phys_page, &flags);
        }

        mmio::sync_barrier();

        #[cfg(feature = "trace_kernel_heap_pages")]
        info!("Allocated {} supervisor stack pages", nb);
    }
}

/**
 * Add memory to the stack until the given address is valid.
 * Panics if there are too many (16) pages added at once, or if memory is
 * exhausted.
 */
pub fn grow_svc_stack(addr: usize) {
    let page = PageId::from(addr);
    let last_stack_page = unsafe { LAST_STACK_PAGE };

    let nb_pages_to_add = last_stack_page.0 - page.0;
    if nb_pages_to_add > 16 {
        panic!("Trying to add too many pages at once in supervisor stack");
    }

    add_svc_stack_pages(nb_pages_to_add);
}

/**
 * Add heap memory for the kernel.
 * Kernel heap memory is mapped between 0x5000_0000 and 0x6FFF_FFFF.
 * This function returns the identifier of the first allocated page.
 * It panics if the requested memory goes above 0x6FFF_FFFF.
 */
pub unsafe fn reserve_heap_pages(nb: usize) -> PageId {
    let first_allocated_page = LAST_HEAP_PAGE;
    for _ in 0..nb {
        if LAST_HEAP_PAGE.0 >= STACK_PAGE_LIMIT.0 {
            panic!("Kernel heap exceeded its maximum size")
        }

        let phys_page = physical_alloc::allocate_page();

        let flags = RegionFlags {
            execute: false,
            global: true,
            shareable: true,
            access: RegionAccess::KernelOnly,
            attributes: RegionAttribute::WriteAllocate,
        };

        KERNEL_SECTION_TABLE.register_page(LAST_HEAP_PAGE, phys_page, &flags);

        LAST_HEAP_PAGE.0 += 1;
    }

    mmio::sync_barrier();

    #[cfg(feature = "trace_kernel_heap_pages")]
    info!(
        "Allocated {} kernel heap pages at {}",
        nb, first_allocated_page
    );

    first_allocated_page
}

pub unsafe fn free_heap_pages(nb: usize) {
    for _ in 0..nb {
        if LAST_HEAP_PAGE.0 <= FIRST_HEAP_PAGE.0 {
            panic!("Cannot free empty kernel heap")
        }
        LAST_HEAP_PAGE.0 -= 1;

        let paddr =
            translate_addr(LAST_HEAP_PAGE.to_addr()).expect("Kernel heap page already deallocated");
        KERNEL_SECTION_TABLE.unregister_page(LAST_HEAP_PAGE);
        cache::tlb::invalidate_page(LAST_HEAP_PAGE);
        physical_alloc::deallocate_page(PageId(paddr / PAGE_SIZE));
    }

    #[cfg(feature = "trace_kernel_heap_pages")]
    info!("Deallocated {} kernel heap pages", nb);
}

pub fn translate_addr(vaddr: usize) -> Option<usize> {
    unsafe { KERNEL_SECTION_TABLE.translate_addr(vaddr) }
}
