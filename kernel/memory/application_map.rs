/*!
 * The application memory map is organized as follows:
 * 0x8000_0000 - 0x9FFF_FFFF: Application code, ELF loader place data here
 * 0xA000_0000 - 0xDFFF_FFFF: Application heap, growing up
 * 0xE000_0000 - 0xFFFF_FFFF: Application stack, growing down
 *
 * Each application has one so each map should be mostly empty.
 */

use alloc::boxed::Box;
use memory::*;
use memory::mmu::*;
use drivers::mmio;
use core::ptr::NonNull;

pub struct ApplicationMap
{
    section_table: Box<SectionTable>,
    last_stack_page: PageId,
    last_heap_page: PageId,
    asid: Option<u8>
}

#[derive(Debug)]
pub enum AppMapError
{
    InvalidProgramAddress,
    StackLimitReached,
    HeapLimitReached,
    HeapEmpty,
    HeapPageAlreadyDeallocated,
}

const FIRST_PRGM_PAGE : PageId = PageId(0x800_00);
const FIRST_HEAP_PAGE : PageId = PageId(0xA00_00);
const LAST_STACK_PAGE : PageId = PageId(0xE00_00);
const AFTER_END_PAGE : PageId = PageId(0x1000_00);

static mut LAST_ASID: u8 = 0;
static mut ASID_MAPS: [Option<NonNull<ApplicationMap>>; 256] = [None; 256];
static mut ACTIVE_MAP: Option<NonNull<ApplicationMap>> = None;

impl ApplicationMap
{
    pub fn new() -> ApplicationMap
    {
        ApplicationMap { section_table: Box::new(mmu::SectionTable::new()),
            last_stack_page: AFTER_END_PAGE, last_heap_page: FIRST_HEAP_PAGE,
            asid: None }
    }

    /// Use the application map for the current core
    pub fn activate(&mut self)
    {
        let asid = match self.asid
        {
            Some(asid) => asid,
            None => unsafe
            {
                // TODO: Manage the multicore case
                let asid = LAST_ASID;
                if let Some(mut old_map) = ASID_MAPS[asid as usize]
                {
                    old_map.as_mut().asid = None;
                    cache::tlb::invalidate_asid(asid);
                }
                LAST_ASID = LAST_ASID.wrapping_add(1);

                ASID_MAPS[asid as usize] = Some(NonNull::new_unchecked(self));
                self.asid = Some(asid);
                asid
            }
        };

        unsafe
        {
            ACTIVE_MAP = Some(NonNull::new_unchecked(self));

            let translation_table = &*self.section_table;
            mmu::set_application_table(translation_table, asid as u32);
        }

    }

    pub fn register_prgm_page(&mut self, vaddr_base: PageId, executable: bool,
                              writable: bool) -> Result<(), AppMapError>
    {
        if vaddr_base.0 < FIRST_PRGM_PAGE.0 || vaddr_base.0 >= FIRST_HEAP_PAGE.0
        {
            return Err(AppMapError::InvalidProgramAddress);
        }

        let phys_page = physical_alloc::allocate_page();

        let flags = RegionFlags { execute: executable, global: false,
            shareable: true, access: if writable { RegionAccess::Full }
                else { RegionAccess::ReadOnlyKernelWrite },
            attributes: RegionAttribute::WriteAllocate };

        self.section_table.register_page(vaddr_base.to_lower(), phys_page, &flags);

        #[cfg(feature = "trace_app_pages")]
        info!("Allocated application progam page at {}", vaddr_base);

        mmio::sync_barrier();
        Ok(())
    }

    pub fn add_stack_page(&mut self) -> Result<(), AppMapError>
    {
        if self.last_stack_page.0 <= LAST_STACK_PAGE.0
        {
            return Err(AppMapError::StackLimitReached);
        }

        self.last_stack_page.0 -= 1;

        let phys_page = physical_alloc::allocate_page();

        let flags = RegionFlags { execute: false, global: false,
            shareable: true, access: RegionAccess::Full,
            attributes: RegionAttribute::WriteAllocate };

        self.section_table.register_page(self.last_stack_page.to_lower(), phys_page, &flags);

        mmio::sync_barrier();
        Ok(())
    }

    /**
    * Add heap memory for the application.
    * Application heap memory is mapped between 0xA000_0000 and 0xDFFF_FFFF.
    * This function returns the identifier of the first allocated page.
    * It returns HeapLimitReached if the requested memory goes above 0xDFFF_FFFF.
    */
    pub fn reserve_heap_pages(&mut self, nb: usize) -> Result<PageId, AppMapError>
    {
        let first_allocated_page = self.last_heap_page;
        for _ in 0 .. nb
        {
            if self.last_heap_page.0 >= 0xE00_00
            {
                return Err(AppMapError::HeapLimitReached);
            }

            let phys_page = physical_alloc::allocate_page();

            let flags = RegionFlags { execute: false, global: false,
                shareable: true, access: RegionAccess::Full,
                attributes: RegionAttribute::WriteAllocate };

            self.section_table.register_page(self.last_heap_page.to_lower(), phys_page, &flags);

            self.last_heap_page.0 += 1;
        }

        mmio::sync_barrier();

        #[cfg(feature = "trace_app_pages")]
        info!("Allocated {} application heap pages at {}", nb, first_allocated_page);

        Ok(first_allocated_page)
    }

    pub unsafe fn free_heap_pages(&mut self, nb: usize) -> Result<(), AppMapError>
    {
        for _ in 0 .. nb
        {
            if self.last_heap_page.0 <= FIRST_HEAP_PAGE.0
            {
                return Err(AppMapError::HeapEmpty);
            }
            self.last_heap_page.0 -= 1;

            let ttbl_addr = self.last_heap_page.to_lower().to_addr();
            let paddr = self.section_table.translate_addr(ttbl_addr)
                .ok_or(AppMapError::HeapPageAlreadyDeallocated)?;

            self.section_table.unregister_page(self.last_heap_page.to_lower());
            if let Some(asid) = self.asid
            {
                cache::tlb::invalidate_asid_page(asid, self.last_heap_page);
            }

            physical_alloc::deallocate_page(PageId(paddr as usize / PAGE_SIZE));
        }

        #[cfg(feature = "trace_app_pages")]
        info!("Deallocated {} application heap pages", nb);

        mmio::sync_barrier();
        Ok(())
    }
}

impl Drop for ApplicationMap
{
    fn drop(&mut self)
    {
        #[cfg(feature = "trace_app_pages")]
        info!("Dropped application map");

        // When the application map is destroyed free all the pages.
        for page in (FIRST_PRGM_PAGE.0 .. self.last_heap_page.0)
                    .chain(self.last_stack_page.0 .. AFTER_END_PAGE.0)
        {
            let ttbl_addr = ((page - 0x800_00) * PAGE_SIZE) as *mut u8;
            if let Some(paddr) = self.section_table.translate_addr(ttbl_addr)
            {
                physical_alloc::deallocate_page(PageId(paddr as usize / PAGE_SIZE));
            }
        }

        unsafe
        {
            if ACTIVE_MAP == Some(NonNull::new_unchecked(self))
            {
                mmu::disable_application_table();
                ACTIVE_MAP = None;
            }

            if let Some(asid) = self.asid
            {
                cache::tlb::invalidate_asid(asid);
                ASID_MAPS[asid as usize] = None;
            }
        }

        mmio::sync_barrier();
    }
}
