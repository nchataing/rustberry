const PAGE_SIZE : usize = 0x1000;
const SECTION_SIZE : usize = 0x10_0000;
const PAGE_BY_SECTION : usize = SECTION_SIZE / PAGE_SIZE;

const MEM_SIZE_MAX : usize = 0x4000_0000; // 1 Go
const NUM_SECTION_MAX : usize = MEM_SIZE_MAX / SECTION_SIZE;
const NUM_PAGES_MAX : usize = MEM_SIZE_MAX / PAGE_SIZE;

use core::fmt;

/// Section identifier (id below 0x400 are physical section identifiers)
#[derive(Clone, Copy)]
pub struct SectionId(usize);

impl SectionId
{
    pub fn to_addr(self) -> *mut u8
    {
        (self.0 * SECTION_SIZE) as *mut u8
    }
}

impl fmt::Display for SectionId
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "S{:#x}", self.0)
    }
}

/// Page identifier (id below 0x4_0000 are physical page identifiers)
#[derive(Clone, Copy)]
pub struct PageId(usize);

impl PageId
{
    pub fn to_addr(self) -> *mut u8
    {
        (self.0 * PAGE_SIZE) as *mut u8
    }
}

impl fmt::Display for PageId
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "P{:#x}", self.0)
    }
}

pub mod pages;
pub mod mmu;
pub mod map;
