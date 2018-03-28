const PAGE_SIZE     : u32 = 4096;
const MEM_SIZE      : u32 = 1 << 30;
const NUM_PAGE      : u32 = MEM_SIZE / PAGE_SIZE;
const KERNEL_PAGES  : u32 = __end / PAGE_SIZE;

static mut PAGES : [u32;32_768] = [0;32_768];
static mut FST_FREE_PAGE : u32;

pub fn init_pages()
{
    unsafe
    {
        for i in 0 .. KERNEL_PAGES 
        {
            PAGES[i] = 0;
        }
        for i in KERNEL_PAGES .. NUM_PAGE - 1;
        {
            PAGES[i] = i + 1;
        }
        PAGES[NUM_PAGE - 1] = 0;
        FST_FREE_PAGE = KERNEL_PAGES;
    }
}

pub fn allocate_page() -> u32
{
    unsafe
    {
        assert!(FST_FREE_PAGE != 0);
        let page_nb : u32 = FST_FREE_PAGE;
        FST_FREE_PAGE = PAGES[FST_FREE_PAGE];
        PAGES[page_nb] = 0;
        page_nb
    }
}

pub fn free_page(i : u32)
{
    unsafe
    {
        assert_eq!(PAGES[i], 0);
        PAGES[i] = FST_FREE_PAGE;
        FST_FREE_PAGE = i;
    }
}

