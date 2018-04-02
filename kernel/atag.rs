#[repr(u32)]
#[derive(PartialEq)]
enum Tag
{
    None      = 0x00000000,
    Core      = 0x54410001,
    Mem       = 0x54410002,
    VideoText = 0x54410003,
    RamDisk   = 0x54410004,
    Initrd2   = 0x54420005,
    Serial    = 0x54410006,
    Revision  = 0x54410007,
    VideoLfb  = 0x54410008,
    CmdLine   = 0x54410009,
}

#[repr(C)]
struct Header
{
    size: isize,
    tag: Tag,
}

#[repr(C)]
struct Mem
{
    header: Header,
    size: usize,
    start: usize
}

const ATAG_BASE : *const Header = 0x100 as *const Header;

pub fn get_mem_size() -> usize
{
    #[cfg(feature = "no_atags")]
    return 1 << 28;

    unsafe
    {
        let mut tag = ATAG_BASE;
        while (*tag).tag != Tag::None
        {
            if (*tag).tag == Tag::Mem
            {
                let mem_tag = tag as *const Mem;
                return (*mem_tag).size;
            }
            tag = (tag as *const usize).offset((*tag).size) as *const Header;
        }
        return 0;
    }
}
