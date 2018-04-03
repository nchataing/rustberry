const PAGE_SIZE : usize = 0x1000;
const SECTION_SIZE : usize = 0x10_0000;
const PAGE_BY_SECTION : usize = SECTION_SIZE / PAGE_SIZE;

const MEM_SIZE_MAX : usize = 0x4000_0000; // 1 Go
const NUM_SECTION_MAX : usize = MEM_SIZE_MAX / SECTION_SIZE;
const NUM_PAGES_MAX : usize = MEM_SIZE_MAX / PAGE_SIZE;

pub mod pages;
pub mod mmu;
