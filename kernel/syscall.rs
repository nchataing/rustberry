use core::{str,slice};
use process::{RegisterContext, ProcessState, ChildEvent};
use scheduler;
use timer;
use io::SeekFrom;
use filesystem::{Dir, virtualfs};

pub fn read(reg_ctx: &mut RegisterContext)
{
    if let Some(process) = scheduler::current_process()
    {
        // Prevent write inside kernel space
        if reg_ctx.r1 < 0x8000_0000 || reg_ctx.r2 >= 0x8000_0000 ||
           reg_ctx.r1.overflowing_add(reg_ctx.r2).1
        {
            reg_ctx.r0 = 0;
            return;
        }

        let buf = unsafe { slice::from_raw_parts_mut(reg_ctx.r1 as *mut u8,
                                                     reg_ctx.r2 as usize) };

        match process.file_descriptors.get_mut(reg_ctx.r0 as usize)
        {
            Some(ref mut file) =>
            {
                match file.read(buf)
                {
                    Ok(bytes_read) => reg_ctx.r0 = bytes_read as u32,
                    Err(err) =>
                    {
                        warn!("{}: error reading file {}: {:?}",
                            process.name, reg_ctx.r0, err);
                        reg_ctx.r0 = 0;
                    }
                }
            }
            None => reg_ctx.r0 = 0,
        }
    }
}

pub fn write(reg_ctx: &mut RegisterContext)
{
    if let Some(process) = scheduler::current_process()
    {
        // Prevent read from kernel space
        if reg_ctx.r1 < 0x8000_0000 || reg_ctx.r2 >= 0x8000_0000 ||
           reg_ctx.r1.overflowing_add(reg_ctx.r2).1
        {
            reg_ctx.r0 = 0;
            return;
        }

        let buf = unsafe { slice::from_raw_parts(reg_ctx.r1 as *mut u8,
                                                 reg_ctx.r2 as usize) };

        match process.file_descriptors.get_mut(reg_ctx.r0 as usize)
        {
            Some(ref mut file) =>
            {
                match file.write(buf)
                {
                    Ok(written_bytes) => reg_ctx.r0 = written_bytes as u32,
                    Err(err) =>
                    {
                        warn!("{}: error writing file {}: {:?}",
                            process.name, reg_ctx.r0, err);
                        reg_ctx.r0 = 0;
                    }
                }
            }
            None => reg_ctx.r0 = 0,
        }
    }
}

pub fn open(reg_ctx: &mut RegisterContext)
{
    if let Some(process) = scheduler::current_process()
    {
        // Prevent read from kernel space
        if reg_ctx.r0 < 0x8000_0000 || reg_ctx.r1 >= 0x8000_0000 ||
           reg_ctx.r0.overflowing_add(reg_ctx.r1).1
        {
            reg_ctx.r0 = (-2i32) as u32;
            return;
        }

        let path_bytes =  unsafe {
            slice::from_raw_parts(reg_ctx.r0 as *const u8, reg_ctx.r1 as usize) };
        let path = str::from_utf8(path_bytes).unwrap_or("");
        match virtualfs::get_root().get_file(path)
        {
            Ok(file) =>
            {
                let descr = process.file_descriptors.insert(file);
                reg_ctx.r0 = descr as u32;
            }
            Err(e) =>
            {
                warn!("{}: cannot open file {}: {:?}", process.name, path, e);
                // TODO: Better error desambiguation
                reg_ctx.r0 = (-1i32) as u32;
            }
        }
    }
}

pub fn close(reg_ctx: &mut RegisterContext)
{
    if let Some(process) = scheduler::current_process()
    {
        process.file_descriptors.remove(reg_ctx.r0 as usize);
    }
}

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

pub fn seek(reg_ctx: &mut RegisterContext)
{
    if let Some(process) = scheduler::current_process()
    {
        let seek_from;
        match reg_ctx.r1
        {
            0 => seek_from = SeekFrom::Start((reg_ctx.r2 as u64) << 32 | reg_ctx.r3 as u64),
            1 => seek_from = SeekFrom::End(((reg_ctx.r2 as u64) << 32 | reg_ctx.r3 as u64) as i64),
            2 => seek_from = SeekFrom::Current(((reg_ctx.r2 as u64) << 32 | reg_ctx.r3 as u64) as i64),
            _ =>
            {
                error!("{}: Invalid seek operation", process.name);
                exit(105);
                return;
            }
        }

        match process.file_descriptors.get_mut(reg_ctx.r0 as usize)
        {
            Some(ref mut file) =>
            {
                match file.seek(seek_from)
                {
                    Ok(offset) =>
                    {
                        reg_ctx.r0 = (offset >> 32) as u32;
                        reg_ctx.r1 = offset as u32;
                    }
                    Err(err) =>
                    {
                        warn!("{}: error seeking file {}: {:?}",
                            process.name, reg_ctx.r0, err);
                        reg_ctx.r0 = 0;
                        reg_ctx.r1 = 0;
                    }
                }
            }
            None =>
            {
                reg_ctx.r0 = 0;
                reg_ctx.r1 = 0;
            }
        }
    }
}

/*pub fn spawn(reg_ctx: &mut RegisterContext)
{
    unimplemented!()
}*/
