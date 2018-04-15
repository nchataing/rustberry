const PAGE_SIZE : usize = 0x1000;
const SECTION_SIZE : usize = 0x10_0000;
const PAGE_BY_SECTION : usize = SECTION_SIZE / PAGE_SIZE;

const MEM_SIZE_MAX : usize = 0x4000_0000; // 1 Go
const NUM_SECTION_MAX : usize = MEM_SIZE_MAX / SECTION_SIZE;
const NUM_PAGES_MAX : usize = MEM_SIZE_MAX / PAGE_SIZE;

use core::fmt;
macro_rules! hex_display
{
    ($x: ident, $prefix: expr) =>
    {
        impl fmt::Display for $x
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
            {
                write!(f, "{}{:#x}", $prefix, self.0)
            }
        }
    }
}

/**
 * Physical section identifier.
 * Multiply by SECTION_SIZE to get the first physical address.
 */
#[derive(Clone, Copy)]
pub struct PhysSectionId(usize);

hex_display!(PhysSectionId, "@S");

impl PhysSectionId
{
    pub fn to_addr(self) -> PhysAddr
    {
        PhysAddr(self.0 * SECTION_SIZE)
    }
}

/**
 * Virtual section identifier.
 * Multiply by SECTION_SIZE to get the first virtual address.
 */
#[derive(Clone, Copy)]
pub struct VirtSectionId(usize);

hex_display!(VirtSectionId, "S");

impl VirtSectionId
{
    pub fn to_addr(self) -> VirtAddr
    {
        (self.0 * SECTION_SIZE) as VirtAddr
    }
}

/**
 * Physical page identifier.
 * Multiply by PAGE_SIZE to get the first physical address.
 */
#[derive(Clone, Copy)]
pub struct PhysPageId(usize);

hex_display!(PhysPageId, "@P");

impl PhysPageId
{
    pub fn to_addr(self) -> PhysAddr
    {
        PhysAddr(self.0 * PAGE_SIZE)
    }
}

/**
 * Virtual page identifier.
 * Multiply by PAGE_SIZE to get the first virtual address.
 */
#[derive(Clone, Copy)]
pub struct VirtPageId(usize);

hex_display!(VirtPageId, "P");

impl VirtPageId
{
    pub fn to_addr(self) -> VirtAddr
    {
        (self.0 * PAGE_SIZE) as VirtAddr
    }
}

/// Physical address
#[derive(Clone, Copy)]
pub struct PhysAddr(usize);

hex_display!(PhysAddr, "@");

/// Virtual address (equivalent to a standard raw pointer)
pub type VirtAddr = *mut u8;

mod pages;
mod mmu;
mod map;

#[no_mangle]
pub extern fn memory_init()
{
    map::init();
}

pub fn init()
{
    map::remove_temporary();
    pages::init();
}
