use process::{RegisterContext, ProcessState, ChildEvent};
use scheduler;
use timer;

/*pub fn read(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

pub fn write(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

pub fn open(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

pub fn close(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}*/

pub fn exit(exit_code: u32)
{
    if let Some(pid) = scheduler::current_pid()
    {
        let exited_process = scheduler::remove_process(pid).unwrap();
        scheduler::send_child_event(exited_process.parent_pid,
                                    ChildEvent { pid, exit_code });
    }
}

pub fn kill(reg_ctx: &mut RegisterContext)
{
    match scheduler::remove_process(reg_ctx.r0 as usize)
    {
        Some(killed_process) =>
        {
            scheduler::send_child_event(killed_process.parent_pid,
                ChildEvent { pid: killed_process.pid, exit_code: 137 });
            reg_ctx.r0 = 0;
        }
        None => reg_ctx.r0 = 1,
    }
}

pub fn reserve_heap_pages(reg_ctx: &mut RegisterContext)
{
    if let Some(current_process) = scheduler::current_process()
    {
        let nb_pages = reg_ctx.r0 as isize;
        if nb_pages >= 0
        {
            match current_process.memory_map.reserve_heap_pages(nb_pages as usize)
            {
                Ok(page_id) => reg_ctx.r0 = page_id.to_addr() as u32,
                Err(err) =>
                {
                    error!("Memory allocation failure: {:?}", err);
                    exit(102);
                }
            }
        }
        else
        {
            match current_process.memory_map.free_heap_pages((-nb_pages) as usize)
            {
                Ok(()) => reg_ctx.r0 = 0,
                Err(err) =>
                {
                    error!("Memory deallocation failure: {:?}", err);
                    exit(102);
                }
            }
        }
    }
}

pub fn sleep(reg_ctx: &mut RegisterContext)
{
    let micro_secs = reg_ctx.r0.saturating_mul(1000);
    if micro_secs == 0 { return; }

    if let Some(pid) = scheduler::current_pid()
    {
        let process = scheduler::get_process(pid).unwrap();
        process.state = ProcessState::WaitingTimer;
        timer::add_wakeup_event(pid, micro_secs);
        scheduler::suspend_process(pid);
    }
}

pub fn wait_children(reg_ctx: &mut RegisterContext)
{
    if let Some(pid) = scheduler::current_pid()
    {
        let process = scheduler::get_process(pid).unwrap();
        match process.child_events.pop()
        {
            Some(child_event) =>
            {
                reg_ctx.r0 = child_event.pid as u32;
                reg_ctx.r1 = child_event.exit_code;
            }
            None =>
            {
                process.state = ProcessState::WaitingChildren;
                scheduler::suspend_process(pid);
            }
        }
    }
}

/*pub fn new_pipe(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

pub fn spawn(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}*/
