use drivers::mmio;
use mem::*;
use core::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum RegionAttribute
{
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
pub enum RegionAccess
{
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

pub struct RegionFlags
{
    pub execute: bool,
    pub global: bool,
    pub shareable: bool,
    pub access: RegionAccess,
    pub attributes: RegionAttribute,
}

pub trait SectionTable<'a>
{
    fn set_entry(&mut self, index: usize, value: usize);

    fn unregister(&mut self, vaddr_base: VirtSectionId)
    {
        self.set_entry(vaddr_base.0, 0);
    }

    fn register_section(&mut self, vaddr_base: VirtSectionId,
                        paddr_base: PhysSectionId, flags: &RegionFlags,
                        kernel_execute: bool)
    {
        let mut entry = (paddr_base.0 << 20) | (1 << 1);
        if !flags.execute { entry |= 1 << 4; }
        if !flags.global { entry |= 1 << 17; }
        if flags.shareable { entry |= 1 << 16; }
        entry |= (flags.access as usize & 0b011) << 10;
        entry |= (flags.access as usize & 0b100) << (15-2);
        entry |= (flags.attributes as usize & 0b00011) << 2;
        entry |= (flags.attributes as usize & 0b11100) << (12-2);
        if !kernel_execute { entry |= 1 << 0; }

        self.set_entry(vaddr_base.0, entry);
    }

    fn register_page_table(&mut self, vaddr_base: VirtSectionId,
                           page_table: &'a PageTable,
                           kernel_execute: bool)
    {
        let mut entry = page_table as *const PageTable as usize | (1 << 0);
        if !kernel_execute { entry |= 1 << 2; }
        self.set_entry(vaddr_base.0, entry);
    }
}

/**
 * Full section table for the kernel mappings. It maps all virtual addresses,
 * and can be used alongside an ApplicationSectionTable (in this case, it is
 * only used for addresses between 0x4000_0000 and 0xFFFF_FFFF).
 */
#[repr(C, align(0x2000))]
pub struct KernelSectionTable<'a>
{
    ttbl: [usize; 0x1000],
    phantom: PhantomData<&'a PageTable>
}

impl<'a> SectionTable<'a> for KernelSectionTable<'a>
{
    fn set_entry(&mut self, index: usize, value: usize)
    {
        self.ttbl[index] = value;
    }
}

impl<'a> KernelSectionTable<'a>
{
    pub const fn new() -> KernelSectionTable<'a>
    {
        KernelSectionTable { ttbl: [0; 0x1000], phantom: PhantomData }
    }
}

/**
 * Small section table mapping addresses from 0x0000_0000 to 0x3FFF_FFFF,
 * used for application-wide mappings.
 */
#[repr(C, align(0x1000))]
pub struct ApplicationSectionTable<'a>
{
    ttbl: [usize; 0x400],
    phantom: PhantomData<&'a PageTable>
}

impl<'a> SectionTable<'a> for ApplicationSectionTable<'a>
{
    fn set_entry(&mut self, index: usize, value: usize)
    {
        self.ttbl[index] = value;
    }
}

impl<'a> ApplicationSectionTable<'a>
{
    pub const fn new() -> ApplicationSectionTable<'a>
    {
        ApplicationSectionTable { ttbl: [0; 0x400], phantom: PhantomData }
    }
}

/**
 * Page table that can be used inside a section table
 */
#[repr(C, align(0x400))]
pub struct PageTable
{
    ttbl: [usize; 0x100]
}

impl PageTable
{
    pub const fn new() -> PageTable
    {
        PageTable { ttbl: [0; 0x100] }
    }

    pub fn unregister(&mut self, vaddr_offset: VirtPageId)
    {
        self.ttbl[vaddr_offset.0] = 0;
    }

    pub fn register_page(&mut self, vaddr_offset: VirtPageId,
                         paddr_base: PhysPageId, flags: &RegionFlags)
    {
        let mut entry = (paddr_base.0 << 12) | (1 << 1);
        if !flags.execute { entry |= 1 << 0; }
        if !flags.global { entry |= 1 << 11; }
        if flags.shareable { entry |= 1 << 10; }
        entry |= (flags.access as usize & 0b011) << 4;
        entry |= (flags.access as usize & 0b100) << (9-2);
        entry |= (flags.attributes as usize & 0b011) << 2;
        entry |= (flags.attributes as usize & 0b100) << (6-2);

        self.ttbl[vaddr_offset.0] = entry;
    }
}

coproc_reg!
{
    TTBR0 : p15, c2, 0, c0, 0;
    TTBR1 : p15, c2, 0, c0, 1;
    TTBCR : p15, c2, 0, c0, 2;
    DACR  : p15, c3, 0, c0, 0;
}

pub unsafe fn setup_kernel_table(ttbl: &'static KernelSectionTable)
{
    use system_control;
    use system_control::Features;

    system_control::disable(Features::MMU | Features::CACHE |
                            Features::BRANCH_PREDICTION |
                            Features::INSTRUCTION_CACHE | Features::TEX_REMAP |
                            Features::ACCESS_FLAG);

    system_control::wipe_instr_cache();
    system_control::wipe_branch_predictor();
    system_control::wipe_tlb();
    mmio::sync_barrier();

    let translation_table = ttbl as *const KernelSectionTable;
    TTBCR::write(2); // Cut between TTBR0 and TTBR1 at 0x4000_0000
    TTBR0::write(translation_table as u32 | 0b1001010);
    TTBR1::write(translation_table as u32 | 0b1001010);
    DACR::write(1); // Use domain 0 only with access check

    system_control::enable(Features::MMU | Features::CACHE |
                           Features::BRANCH_PREDICTION |
                           Features::INSTRUCTION_CACHE |
                           Features::SWP_INSTRUCTION |
                           Features::ALIGNMENT_CHECK);
    mmio::sync_barrier();
}

