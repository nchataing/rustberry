use atag;
use mem::*;

linker_symbol!
{
    static __end;
}

#[derive(Clone, Copy)]
struct Section
{
    free_pages: u16, // 0 -> Full, 256 -> Free, other -> Divided
    next: u16, // Next divided section if Divided, next free section if Free
    prev: u16, // Previous divided section if Divided
}

const FULL_SECTION : Section = Section { free_pages: 0, next: 0, prev: 0 };

static mut SECTIONS : [Section; NUM_SECTION_MAX] = [FULL_SECTION; NUM_SECTION_MAX];
static mut FST_FREE_SECTION : u16 = 0;
static mut FST_DIVIDED_SECTION : u16 = 0;

static mut PAGES : [u16; NUM_PAGES_MAX / 16] = [0; NUM_PAGES_MAX / 16];

pub fn init()
{
    let mem_size = atag::get_mem_size();
    let kernel_sections = (linker_symbol!(__end) - 1) / SECTION_SIZE + 1;
    let num_section = mem_size / SECTION_SIZE;

    unsafe
    {
        // From 0 to kernel_sections : SECTIONS[i] = FULL_SECTION
        for i in kernel_sections .. num_section - 1
        {
            SECTIONS[i].free_pages = 256;
            SECTIONS[i].next = (i+1) as u16;
        }
        SECTIONS[num_section-1].free_pages = 256; // and next = 0
        // Unavailable sections : SECTIONS[i] = FULL_SECTION

        FST_FREE_SECTION = kernel_sections as u16;
        FST_DIVIDED_SECTION = 0;
    }
}

pub fn allocate_section() -> PhysSectionId
{
    unsafe
    {
        assert!(FST_FREE_SECTION != 0);

        let section_nb = FST_FREE_SECTION as usize;
        match SECTIONS[section_nb]
        {
            Section { free_pages: 256, next, .. } =>
            {
                SECTIONS[section_nb].free_pages = 0;
                FST_FREE_SECTION = next;
                // There is no need to update pages here
            }
            _ => panic!("Section already allocated"),
        }
        PhysSectionId(section_nb)
    }
}

pub fn deallocate_section(i : PhysSectionId)
{
    unsafe
    {
        match SECTIONS[i.0]
        {
            Section { free_pages: 0, .. } => (),
            Section { free_pages: 256, .. } =>
                panic!("Deallocating free section {}", i),
            _ => panic!("Deallocating divided section {}", i)
        }
        SECTIONS[i.0].free_pages = 256;
        SECTIONS[i.0].next = FST_FREE_SECTION;
        FST_FREE_SECTION = i.0 as u16;
    }
}

pub fn allocate_page() -> PhysPageId
{
    unsafe
    {
        if FST_DIVIDED_SECTION == 0 && FST_FREE_SECTION == 0
        {
            panic!("No more memory available !");
        }

        if FST_DIVIDED_SECTION == 0
        {
            FST_DIVIDED_SECTION = FST_FREE_SECTION;
            FST_FREE_SECTION = SECTIONS[FST_FREE_SECTION as usize].next;
            SECTIONS[FST_DIVIDED_SECTION as usize].next = 0;
        }

        let mut allocated_page : usize = 0;

        'outer: for page_group in 0 .. 16
        {
            let page_group_id = FST_DIVIDED_SECTION as usize * 16 + page_group;
            let page = &mut PAGES[page_group_id];
            if *page != 0xFFFF
            {
                for i in 0 .. 16
                {
                    if *page & (1 << i) == 0
                    {
                        allocated_page = i + 16 * page_group_id;
                        *page |= 1 << i;

                        let section = &mut SECTIONS[FST_DIVIDED_SECTION as usize];
                        section.free_pages -= 1;
                        if section.free_pages == 0
                        {
                            FST_DIVIDED_SECTION = section.next;
                        }

                        break 'outer;
                    }
                }
            }
        }
        PhysPageId(allocated_page)
    }
}

pub fn deallocate_page(page_id: PhysPageId)
{
    unsafe
    {
        let section_id = (page_id.0 / PAGE_BY_SECTION) as u16;
        let section = &mut SECTIONS[section_id as usize];
        let page_group = &mut PAGES[page_id.0 / 16];
        let page_pos = page_id.0 % 16;

        if *page_group & (1 << page_pos) == 0
        {
            panic!("Page {} is not allocated", page_id);
        }
        *page_group &= !(1 << page_pos);

        section.free_pages += 1;
        if section.free_pages == 1
        {
            section.next = FST_DIVIDED_SECTION;
            SECTIONS[FST_DIVIDED_SECTION as usize].prev = section_id;
            FST_DIVIDED_SECTION = section_id;
        }
        else if section.free_pages == 256
        {
            // Remove the section from the divided section list
            if section_id == FST_DIVIDED_SECTION
            {
                FST_DIVIDED_SECTION = section.next;
            }
            else
            {
                SECTIONS[section.prev as usize].next = section.next;
            }

            // Add it to free section list
            section.next = FST_FREE_SECTION;
            FST_FREE_SECTION = section_id as u16;
        }
    }
}
