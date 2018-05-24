use io::SeekFrom;

#[inline]
pub fn reschedule()
{
    unsafe
    {
        asm!("svc 0" :::: "volatile");
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FileDescriptor(usize);

#[inline]
pub(crate) fn read(file: FileDescriptor, buf: &mut [u8]) -> usize
{
    let read_bytes;
    unsafe
    {
        asm!("svc 1" : "={r0}"(read_bytes) : "{r0}"(file.0),
                       "{r1}"(buf.as_ptr()), "{r2}"(buf.len()));
    }
    read_bytes
}

#[inline]
pub(crate) fn write(file: FileDescriptor, buf: &[u8]) -> usize
{
    let written_bytes;
    unsafe
    {
        asm!("svc 2" : "={r0}"(written_bytes) : "{r0}"(file.0),
                       "{r1}"(buf.as_ptr()), "{r2}"(buf.len()));
    }
    written_bytes
}

/*pub enum OpenFlags
{
    Read = 0x1,
    Write = 0x2,
    Create = 0x4,
    Append = 0x8,
    Truncate = 0x10,
}*/

#[inline]
pub(crate) fn open(path: &str/*, flags: OpenFlags*/) -> Option<FileDescriptor>
{
    let fdesc : i32;
    unsafe
    {
        asm!("svc 3" : "={r0}"(fdesc) : "{r0}"(path.as_ptr()),
                       "{r1}"(path.len())/*, "{r2}"(flags as u32)*/ :: "volatile");
    }
    if fdesc >= 0 { Some(FileDescriptor(fdesc as usize)) }
    else { None }
}

#[inline]
pub(crate) fn close(file: FileDescriptor)
{
    unsafe
    {
        asm!("svc 4" :: "{r0}"(file.0) :: "volatile");
    }
}

#[inline]
pub fn exit(exit_code: u32) -> !
{
    unsafe
    {
        asm!("svc 5" :: "{r0}"(exit_code) :: "volatile");
    }
    loop {} // We should never come here !
}

#[inline]
pub fn kill(pid: usize) -> bool
{
    let result: u32;
    unsafe
    {
        asm!("svc 6" : "={r0}"(result) : "{r0}"(pid) :: "volatile");
    }
    result != 0
}

#[inline]
pub(crate) unsafe fn reserve_heap_pages(nb: isize) -> usize
{
    let first_allocated;
    asm!("svc 7" : "={r0}"(first_allocated) : "{r0}"(nb) :: "volatile");
    first_allocated
}

#[inline]
pub fn sleep(msec: usize)
{
    unsafe
    {
        asm!("svc 8" :: "{r0}"(msec) :: "volatile");
    }
}

pub struct ChildEvent
{
    pub pid: usize,
    pub exit_code: u32,
}

#[inline]
pub fn wait_children() -> ChildEvent
{
    let pid;
    let exit_code;
    unsafe
    {
        asm!("svc 9" : "={r0}"(pid), "={r1}"(exit_code)
             ::: "volatile");
    }
    ChildEvent { pid, exit_code }
}

#[inline]
pub(crate) fn seek(fdesc: FileDescriptor, seek_pos: SeekFrom) -> u64
{
    let seek_origin;
    let seek_low;
    let seek_high;
    match seek_pos
    {
        SeekFrom::Start(off) =>
        {
            seek_origin = 0;
            seek_low = off as u32;
            seek_high = (off >> 32) as u32;
        }
        SeekFrom::End(off) =>
        {
            seek_origin = 1;
            seek_low = off as u32;
            seek_high = (off as u64 >> 32) as u32;
        }
        SeekFrom::Current(off) =>
        {
            seek_origin = 2;
            seek_low = off as u32;
            seek_high = (off as u64 >> 32) as u32;
        }
    }

    let offset_high : u32;
    let offset_low : u32;
    unsafe
    {
        asm!("svc 10" : "={r0}"(offset_high), "={r1}"(offset_low) :
                        "{r0}"(fdesc.0), "{r1}"(seek_origin),
                        "{r2}"(seek_high), "{r3}"(seek_low) :: "volatile");
    }
    (offset_high as u64) << 32 | offset_low as u64
}

/*#[inline]
pub fn spawn()
{
    unimplemented!() // svc 11
}*/
