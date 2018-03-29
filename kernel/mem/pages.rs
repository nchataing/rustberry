use atag;

const PAGE_SIZE : usize = 4096;
extern
{
    static __bss_start: u8;
    static __bss_end: u8;
    static __end: u8;
}
const MEM_SIZE_MAX : usize = 0x4000_0000; // 1 Go
const NUM_PAGE_MAX : usize = MEM_SIZE_MAX / PAGE_SIZE;

static mut PAGES : [usize;NUM_PAGE_MAX] = [0; NUM_PAGE_MAX];
static mut FST_FREE_PAGE : usize = 0;

pub fn init()
{
    unsafe
    {
        let mem_size = atag::get_mem_size();
        let kernel_pages = (&__end) as *const u8 as usize / PAGE_SIZE;
        let num_page = mem_size / PAGE_SIZE;

        // From 0 to kernel_pages, PAGES[i] = 0
        for i in kernel_pages .. num_page - 1
        {
            PAGES[i] = i + 1;
        }
        // For unavailable memory, PAGES[i] = 0

        FST_FREE_PAGE = kernel_pages;
    }
}

pub fn allocate() -> usize
{
    unsafe
    {
        assert!(FST_FREE_PAGE != 0);
        let page_nb = FST_FREE_PAGE;
        FST_FREE_PAGE = PAGES[FST_FREE_PAGE];
        PAGES[page_nb] = 0;
        page_nb
    }
}

pub fn deallocate(i : usize)
{
    unsafe
    {
        assert_eq!(PAGES[i], 0);
        PAGES[i] = FST_FREE_PAGE;
        FST_FREE_PAGE = i;
    }
}

