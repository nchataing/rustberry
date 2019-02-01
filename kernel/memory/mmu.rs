use super::*;
use drivers::mmio;

#[derive(Clone, Copy)]
pub enum RegionAttribute {
    /// Strongly-ordered (shareable is ignored)
    StronglyOrdered = 0b000,

    /// Shareable device (shareable is ignored)
    Device = 0b001,

    /// Outer and Inner Non-cacheable
    NonCacheable = 0b100,

    /// Outer and Inner Write-Through, no Write-Allocate
    WriteThrough = 0b010,

    /// Outer and Inner Write-Back, no Write-Allocate
    WriteBack = 0b011,

    /// Outer and Inner Write-Back, Write-Allocate
    WriteAllocate = 0b111,
}

#[derive(Clone, Copy)]
pub enum RegionAccess {
    /// All accesses generate Permission faults
    Forbidden = 0b000,

    /// Access only at PL1
    KernelOnly = 0b001,

    /// Writes at PL0 generate Permission faults
    ReadOnlyKernelWrite = 0b010,

    /// Full access
    Full = 0b011,

    /// Read-only, only at PL1
    KernelReadOnly = 0b101,

    /// Read-only at any privilege level
    ReadOnly = 0b111,
}

pub struct RegionFlags {
    pub execute: bool,
    pub global: bool,
    pub shareable: bool,
    pub access: RegionAccess,
    pub attributes: RegionAttribute,
}

/**
 * Section table for virtual address mappings.
 * A section table can be used for the kernel or for an application.
 * If used as the kernel table, it maps the lower half virtual addresses
 * (between 0x0000_0000 and 0x7FFF_FFFF).
 * Else it maps the higher half virtual addresses (0x8000_0000 to 0xFFFF_FFFF)
 * as if all the given SectionId were increased by 0x800.
 */
#[repr(C, align(0x2000))]
pub struct SectionTable {
    ttbl: [usize; 0x800],
}

impl SectionTable {
    pub const fn new() -> SectionTable {
        SectionTable { ttbl: [0; 0x800] }
    }

    pub fn unregister_section(&mut self, vaddr_base: SectionId) {
        self.ttbl[vaddr_base.0] = 0;
    }

    pub fn register_section(
        &mut self,
        vaddr_base: SectionId,
        paddr_base: SectionId,
        flags: &RegionFlags,
        kernel_execute: bool,
    ) {
        let mut entry = (paddr_base.0 << 20) | (1 << 1);
        if !flags.execute {
            entry |= 1 << 4;
        }
        if !flags.global {
            entry |= 1 << 17;
        }
        if flags.shareable {
            entry |= 1 << 16;
        }
        entry |= (flags.access as usize & 0b011) << 10;
        entry |= (flags.access as usize & 0b100) << (15 - 2);
        entry |= (flags.attributes as usize & 0b00011) << 2;
        entry |= (flags.attributes as usize & 0b11100) << (12 - 2);
        if !kernel_execute {
            entry |= 1 << 0;
        }

        self.ttbl[vaddr_base.0] = entry;
    }

    /**
     * Register a page table in the section table.
     * It can only be safely called from outside this module
     * if the section table is never destroyed.
     */
    pub unsafe fn register_page_table(
        &mut self,
        vaddr_base: SectionId,
        page_table: *const PageTable,
        kernel_execute: bool,
    ) {
        let mut entry = page_table as usize | (1 << 0);
        if !kernel_execute {
            entry |= 1 << 2;
        }
        self.ttbl[vaddr_base.0] = entry;
    }

    fn divide_sections(&mut self, vaddr_base: SectionId) -> *mut PageTable {
        let fst_section_id = (vaddr_base.0 / 4) * 4;
        for section in self.ttbl[fst_section_id..fst_section_id + 4].iter() {
            assert!(section & 0b11 == 0)
        }

        let page_addr = physical_alloc::allocate_page().to_addr();
        unsafe {
            for offset in 0..PAGE_SIZE as isize {
                *((page_addr as *mut u8).offset(offset)) = 0
            }

            for section in 0..4 {
                self.register_page_table(
                    SectionId(fst_section_id + section as usize),
                    (page_addr as *const PageTable).offset(section),
                    false,
                );
            }

            (page_addr as *mut PageTable).offset((vaddr_base.0 % 4) as isize)
        }
    }

    pub fn get_page_table(&self, vaddr_base: SectionId) -> Option<*mut PageTable> {
        let entry = self.ttbl[vaddr_base.0];
        if entry & 0b11 == 0b01 {
            Some((entry & 0xffff_fc00) as *mut PageTable)
        } else {
            None
        }
    }

    pub fn unregister_page(&mut self, vaddr_base: PageId) {
        let section_id = SectionId(vaddr_base.0 / PAGE_BY_SECTION);
        let page_table = self
            .get_page_table(section_id)
            .expect("cannot deallocate inside not divided section");

        unsafe { (*page_table).unregister_page(PageId(vaddr_base.0 % PAGE_BY_SECTION)) }
    }

    pub fn register_page(&mut self, vaddr_base: PageId, paddr_base: PageId, flags: &RegionFlags) {
        let section_id = SectionId(vaddr_base.0 / PAGE_BY_SECTION);
        let page_table = match self.get_page_table(section_id) {
            None => self.divide_sections(section_id),
            Some(ptbl) => ptbl,
        };

        unsafe {
            (*page_table).register_page(PageId(vaddr_base.0 % PAGE_BY_SECTION), paddr_base, flags);
        }
    }

    pub fn translate_addr(&self, vaddr: usize) -> Option<usize> {
        let vsection = vaddr / SECTION_SIZE;
        let vpage = (vaddr / PAGE_SIZE) % PAGE_BY_SECTION;

        let entry = self.ttbl[vsection];
        let ppage = match entry & 0b11 {
            0b00 => return None,
            0b01 => {
                let page_table = self.get_page_table(SectionId(vsection)).unwrap();
                unsafe { (*page_table).translate_page(PageId(vpage))?.0 }
            }
            _ => (entry >> 20) * PAGE_BY_SECTION + vpage,
        };
        Some(ppage * PAGE_SIZE + (vaddr % PAGE_SIZE))
    }
}

// Section table should destroy the created page tables when deleted
impl Drop for SectionTable {
    fn drop(&mut self) {
        for entry in self.ttbl.iter().step_by(4) {
            if entry & 0b11 == 0b01 && entry & 0xC00 == 0 {
                physical_alloc::deallocate_page(PageId(entry / PAGE_SIZE));
            }
        }
    }
}

#[repr(C, align(0x400))]
pub struct PageTable {
    ttbl: [usize; 0x100],
}

impl PageTable {
    pub const fn new() -> PageTable {
        PageTable { ttbl: [0; 0x100] }
    }

    pub fn unregister_page(&mut self, vaddr_offset: PageId) {
        self.ttbl[vaddr_offset.0] = 0;
    }

    pub fn register_page(&mut self, vaddr_offset: PageId, paddr_base: PageId, flags: &RegionFlags) {
        let mut entry = (paddr_base.0 << 12) | (1 << 1);
        if !flags.execute {
            entry |= 1 << 0;
        }
        if !flags.global {
            entry |= 1 << 11;
        }
        if flags.shareable {
            entry |= 1 << 10;
        }
        entry |= (flags.access as usize & 0b011) << 4;
        entry |= (flags.access as usize & 0b100) << (9 - 2);
        entry |= (flags.attributes as usize & 0b011) << 2;
        entry |= (flags.attributes as usize & 0b100) << (6 - 2);

        self.ttbl[vaddr_offset.0] = entry;
    }

    pub fn translate_page(&self, vaddr: PageId) -> Option<PageId> {
        let entry = self.ttbl[vaddr.0];
        if entry & 1 << 1 == 0 {
            None
        } else {
            Some(PageId(entry >> 12))
        }
    }
}

coproc_reg! {
    TTBR0 : p15, c2, 0, c0, 0;
    TTBR1 : p15, c2, 0, c0, 1;
    TTBCR : p15, c2, 0, c0, 2;
    DACR  : p15, c3, 0, c0, 0;

    CONTEXTIDR: p15, c13, 0, c0, 1;
}

pub unsafe fn setup_kernel_table(translation_table: *const SectionTable) {
    use crate::system_control;
    use crate::system_control::Features;

    system_control::disable_features(
        Features::MMU
            | Features::CACHE
            | Features::BRANCH_PREDICTION
            | Features::INSTRUCTION_CACHE
            | Features::TEX_REMAP
            | Features::ACCESS_FLAG,
    );

    cache::invalidate_instr_cache();
    cache::invalidate_branch_predictor();
    cache::tlb::invalidate_all();
    mmio::sync_barrier();

    TTBCR::write(1 | 1 << 5); // Cut between TTBR0 and TTBR1 at 0x8000_0000
    TTBR0::write(translation_table as u32 | 0b1001010);
    DACR::write(1); // Use domain 0 only with access check

    system_control::enable_features(
        Features::MMU
            | Features::CACHE
            | Features::BRANCH_PREDICTION
            | Features::INSTRUCTION_CACHE
            | Features::SWP_INSTRUCTION
            | Features::ALIGNMENT_CHECK,
    );
    mmio::sync_barrier();
}

pub unsafe fn set_application_table(translation_table: *const SectionTable, cxid: u32) {
    // Disable higher half translation table
    disable_application_table();

    // Set the context id, note that lower 8 bits are used as application space id
    CONTEXTIDR::write(cxid);

    // Set the translation table
    TTBR1::write(translation_table.offset(-1) as u32 | 0b1001010);

    // Reenable higher half translation table
    mmio::instr_barrier();
    TTBCR::reset_bits(1 << 5);
}

pub unsafe fn disable_application_table() {
    TTBCR::set_bits(1 << 5);
    mmio::instr_barrier();
}
