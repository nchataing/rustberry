pub const PAGE_SIZE : usize = 0x1000;
pub const SECTION_SIZE : usize = 0x10_0000;
pub const PAGE_BY_SECTION : usize = SECTION_SIZE / PAGE_SIZE;

const MEM_SIZE_MAX : usize = 0x3E00_0000; // ~ 1 Go
const NUM_SECTION_MAX : usize = MEM_SIZE_MAX / SECTION_SIZE;
const NUM_PAGES_MAX : usize = MEM_SIZE_MAX / PAGE_SIZE;

const FIRST_VIRTUAL_SECTION : usize = 0x401;

use core::fmt;

/// Section identifier (id below 0x401 are physical section identifiers)
#[derive(Clone, Copy, Debug)]
pub struct SectionId(pub usize);

impl SectionId
{
    pub const fn to_addr(self) -> usize
    {
        self.0 * SECTION_SIZE
    }

    pub const fn to_page(self) -> PageId
    {
        PageId(self.0 * PAGE_BY_SECTION)
    }
}

impl From<usize> for SectionId
{
    fn from(addr: usize) -> SectionId
    {
        SectionId(addr / SECTION_SIZE)
    }
}

impl From<PageId> for SectionId
{
    fn from(page: PageId) -> SectionId
    {
        SectionId(page.0 / PAGE_BY_SECTION)
    }
}

impl fmt::Display for SectionId
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "S{:#x}", self.0)
    }
}

/// Page identifier (id below 0x401_00 are physical page identifiers)
#[derive(Clone, Copy, Debug)]
pub struct PageId(pub usize);

impl PageId
{
    pub const fn to_addr(self) -> usize
    {
        self.0 * PAGE_SIZE
    }

    pub fn to_lower(self) -> PageId
    {
        assert!(self.0 >= 0x800_00);
        PageId(self.0 - 0x800_00)
    }

    pub fn to_upper(self) -> PageId
    {
        assert!(self.0 < 0x800_00);
        PageId(self.0 + 0x800_00)
    }
}

impl From<usize> for PageId
{
    fn from(addr: usize) -> PageId
    {
        PageId(addr / PAGE_SIZE)
    }
}

impl fmt::Display for PageId
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "P{:#x}", self.0)
    }
}

pub mod physical_alloc;
pub mod cache;
pub mod mmu;
pub mod kernel_map;
pub mod kernel_alloc;
pub mod application_map;
