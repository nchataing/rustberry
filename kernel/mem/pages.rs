use atag;
use mem::pages::Section::*;

const PAGE_SIZE : usize = 4096;
const SECTION_SIZE : usize = 1024 * 1024;
const PAGE_BY_SECTION : usize = SECTION_SIZE / PAGE_SIZE;

extern
{
    static __bss_start: u8;
    static __bss_end: u8;
    static __end: u8;
}

#[derive(Clone, Copy)]
enum Section
{
    Full,
    Free(usize), // Next free section
    Divided(usize,usize), // Nb page left, next divided section
}

const MEM_SIZE_MAX : usize = 0x4000_0000; // 1 Go
const NUM_SECTION_MAX : usize = MEM_SIZE_MAX / SECTION_SIZE;
const NUM_PAGES_MAX : usize = MEM_SIZE_MAX / PAGE_SIZE;

static mut SECTIONS : [Section; NUM_SECTION_MAX] = [Full; NUM_SECTION_MAX];
static mut FST_FREE_SECTION : usize = 0;
static mut FST_DIVIDED_SECTION : usize = 0;

static mut PAGES : [u16; NUM_PAGES_MAX / 16] = [0xFFFF; NUM_PAGES_MAX / 16];

pub fn init()
{
    unsafe
    {
        let mem_size = 1 << 28;//atag::get_mem_size();
        let kernel_sections = (&__end) as *const u8 as usize / SECTION_SIZE + 1;
        let num_section = mem_size / SECTION_SIZE;

        // From 0 to kernel_pages, SECTION[i] = Full
        for i in kernel_sections .. num_section - 1
        {
            SECTIONS[i] = Free(i+1);
        }
        // For unavailable memory, SECTION[i] = Full

        FST_FREE_SECTION = kernel_sections;
        FST_DIVIDED_SECTION = 0;
    }
}

pub fn allocate_section() -> usize
{
    unsafe
    {
        assert!(FST_FREE_SECTION != 0);
        let section_nb = FST_FREE_SECTION;
        match SECTIONS[FST_FREE_SECTION]
        {
            Full | Divided(_,_) => panic!("Section already allocated"),
            Free(next) => {
                SECTIONS[FST_FREE_SECTION] = Full;
                FST_FREE_SECTION = next;
                // There is no need to update pages here
            }
        }
        section_nb
    }
}

pub fn deallocate_section(i : usize)
{
    unsafe
    {
        match SECTIONS[i]
        {
            Full => (),
            Free(_) | Divided(_,_) => panic!("")
        }
        SECTIONS[i] = Free(FST_FREE_SECTION);
        FST_FREE_SECTION = i;
    }
}

pub fn allocate_page() -> usize
{
    unsafe
    {
        assert!(FST_DIVIDED_SECTION != 0 || FST_FREE_SECTION != 0);
        if FST_DIVIDED_SECTION == 0 
        {
            FST_DIVIDED_SECTION = FST_FREE_SECTION;
            match SECTIONS[FST_FREE_SECTION]
            {
                Full | Divided(_,_) => panic!("No more memory available!"),
                Free(next) => FST_FREE_SECTION = next
            }
            SECTIONS[FST_DIVIDED_SECTION] = Divided(256,0);
        }

        let mut allocated_page : usize = 0;

        'outer : for page_group in 0 .. 16
        {
            let x = &PAGES[FST_DIVIDED_SECTION * 16 + page_group];
            if *x != 0
            {
                for i in 0 .. 16
                {
                    if x & (1 << i) != 0 
                    {
                        allocated_page = i + 16 * page_group 
                            + PAGE_BY_SECTION * FST_DIVIDED_SECTION;
                        PAGES[FST_DIVIDED_SECTION * 16 + page_group] = x & !(1<< i);
                        match SECTIONS[FST_DIVIDED_SECTION]
                        {
                            Full | Free (_) => panic!("Error\n"),
                            Divided (page_left, next) => {
                                SECTIONS[FST_DIVIDED_SECTION] = 
                                    Divided(page_left-1, next);
                                if page_left - 1 == 0 && next == 0 {
                                    FST_DIVIDED_SECTION = 0;
                                }
                            }
                        }
                        break 'outer;
                    }
                }
            }
        }
        allocated_page
    }
}

pub fn deallocate_page(page_id: usize)
{
    unsafe
    {
        let section_id = page_id / PAGE_BY_SECTION;
        let page_group = page_id / 16;
        let page_pos = page_id % 16;

        match SECTIONS[section_id]
        {
            Full => panic!("Page {:#x} was deallocated in section {:#x}\n", 
                            page_id, section_id),
            Free (_) => panic!("Page {:#x} was never allocated\n", page_id),
            Divided(page_left, next) => {
                if (PAGES[page_group] & (1 << page_pos) != 0)
                {
                    panic!("Page {:#x} is not allocated\n", page_id);
                }
                PAGES[page_group] = PAGES[page_group] | (1 << page_pos);
                let page_left = page_left + 1;
                if page_left == 256
                {
                    SECTIONS[section_id] = Free (FST_FREE_SECTION);
                    FST_FREE_SECTION = section_id;
                }
                else 
                {
                    SECTIONS[section_id] = Divided(page_left, next)
                }
            }
        }
    }
}