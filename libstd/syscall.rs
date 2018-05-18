pub fn reschedule()
{
    unsafe
    {
        asm!("svc 0" :::: "volatile");
    }
}

/*pub fn read()
{
    unimplemented!() // svc 1
}

pub fn write()
{
    unimplemented!() // svc 2
}

pub fn open()
{
    unimplemented!() // svc 3
}

pub fn close()
{
    unimplemented!() // svc 4
}*/

pub fn exit(exit_code: u32) -> !
{
    unsafe
    {
        asm!("svc 5" :: "{r0}"(exit_code) :: "volatile");
    }
    loop {} // We should never come here !
}

pub fn kill(pid: usize) -> bool
{
    let result: u32;
    unsafe
    {
        asm!("svc 6" : "={r0}"(result) : "{r0}"(pid) :: "volatile");
    }
    result != 0
}

pub unsafe fn reserve_heap_pages(nb: isize) -> usize
{
    let first_allocated;
    asm!("svc 7" : "={r0}"(first_allocated) : "{r0}"(nb) :: "volatile");
    first_allocated
}

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

/*pub fn new_pipe()
{
    unimplemented!() // svc 10
}

pub fn spawn()
{
    unimplemented!() // svc 11
}*/
