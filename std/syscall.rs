#[inline]
pub fn reschedule()
{
    unsafe
    {
        asm!("svc 0" :::: "volatile");
    }
}

/*#[inline]
pub fn read()
{
    unimplemented!() // svc 1
}

#[inline]
pub fn write()
{
    unimplemented!() // svc 2
}

#[inline]
pub fn open()
{
    unimplemented!() // svc 3
}

#[inline]
pub fn close()
{
    unimplemented!() // svc 4
}*/

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
pub unsafe fn reserve_heap_pages(nb: isize) -> usize
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

/*#[inline]
pub fn new_pipe()
{
    unimplemented!() // svc 10
}

#[inline]
pub fn spawn()
{
    unimplemented!() // svc 11
}*/
