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

pub fn kill(pid: usize)
{
    unsafe
    {
        asm!("svc 6" :: "{r0}"(pid) :: "volatile");
    }
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

pub struct SignalEvent
{
    pub sig_id: usize,
    pub sender_pid: usize,
}

pub fn wait_signal() -> SignalEvent
{
    let sig_id;
    let sender_pid;
    unsafe
    {
        asm!("svc 9" : "={r0}"(sig_id), "={r1}"(sender_pid)
             ::: "volatile");
    }
    SignalEvent { sig_id, sender_pid }
}

pub fn send_signal(sig_id: usize, pid: usize)
{
    unsafe
    {
        asm!("svc 10" :: "{r0}"(sig_id), "{r1}"(pid) :: "volatile");
    }
}

/*pub fn spawn()
{
    unimplemented!() // svc 11
}*/
