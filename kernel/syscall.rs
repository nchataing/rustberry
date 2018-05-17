use process::RegisterContext;
use scheduler;

#[no_mangle]
pub unsafe extern fn software_interrupt_handler(reg_ctx: &mut RegisterContext)
{
    // We can get the content of lr when the interruption occur as the 5th
    // argument as it was pushed on the stack by the assembly code.
    let syscall_id = *(reg_ctx.pc.offset(-1)) & 0x00ff_ffff;

    match syscall_id
    {
        0 => scheduler::plan_scheduling(),
        /*1 => read(reg_ctx),
        2 => write(reg_ctx),
        3 => open(reg_ctx),
        4 => close(reg_ctx),*/
        5 => exit(),
        6 => kill(reg_ctx),
        7 => reserve_heap_pages(reg_ctx),
        /*8 => sleep(reg_ctx),
        9 => wait_signal(reg_ctx),
        10 => send_signal(reg_ctx),
        11 => spawn(reg_ctx),*/
        _ => warn!("Invalid syscall {}", syscall_id),
    }

    scheduler::check_schedule(reg_ctx);
}

/*fn read(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

fn write(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

fn open(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

fn close(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}*/

fn exit()
{
    match scheduler::current_pid()
    {
        Some(pid) =>
        {
            scheduler::remove_process(pid);
        },
        None => (),
    }
}

fn kill(reg_ctx: &mut RegisterContext)
{
    scheduler::remove_process(reg_ctx.r0 as usize);
}

fn reserve_heap_pages(reg_ctx: &mut RegisterContext)
{
    match scheduler::current_process()
    {
        Some(current_process) =>
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
                        exit();
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
                        exit();
                    }
                }
            }
        },
        None => (),
    }
}

/*fn sleep(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

fn wait_signal(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

fn send_signal(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}

fn spawn(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}*/
