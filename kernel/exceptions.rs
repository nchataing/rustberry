use drivers;
use memory::{kernel_map, application_map};
use process::RegisterContext;
use scheduler;
use syscall;
use system_control;
use system_control::ProcessorMode;

#[no_mangle]
pub extern fn undefined_instruction_handler(reg_ctx: &mut RegisterContext)
{
    let instr_addr = reg_ctx.pc;
    if system_control::get_spsr() & 0b11111 == ProcessorMode::User as u32
    {
        // Error in application code
        if let Some(process) = scheduler::current_process()
        {
            error!("{}: Undefined instruction at {:p}", process.name, instr_addr);
        }
        else
        {
            panic!("Undefined instruction at {:p} while no current process running",
                   instr_addr);
        }
        syscall::exit(103);
        scheduler::check_schedule(reg_ctx);
    }
    else
    {
        // Error in kernel code
        panic!("Undefined kernel instruction at {:p}", instr_addr);
    }
}

#[no_mangle]
pub unsafe extern fn software_interrupt_handler(reg_ctx: &mut RegisterContext)
{
    let syscall_id = *(reg_ctx.pc.offset(-1)) & 0x00ff_ffff;

    match syscall_id
    {
        0 => scheduler::plan_scheduling(),
        1 => syscall::read(reg_ctx),
        2 => syscall::write(reg_ctx),
        /*3 => syscall::open(reg_ctx),
        4 => syscall::close(reg_ctx),*/
        5 => syscall::exit(reg_ctx.r0),
        6 => syscall::kill(reg_ctx),
        7 => syscall::reserve_heap_pages(reg_ctx),
        8 => syscall::sleep(reg_ctx),
        9 => syscall::wait_children(reg_ctx),
        10 => syscall::seek(reg_ctx),
        /*11 => syscall::spawn(reg_ctx),*/
        _ => warn!("Invalid syscall {}", syscall_id),
    }

    scheduler::check_schedule(reg_ctx);
}

fn fault_description(status: u32) -> &'static str
{
    let fault_type = status & 0b1111;

    if status & (1 << 10) == 0
    {
        match fault_type
        {
            0b0001 => "Alignment fault",
            0b0010 => "Debug event",
            0b0011 => "Access flag fault (section)",
            0b0100 => "Instruction cache maintenance fault",
            0b0101 => "Translation fault (section)",
            0b0110 => "Access flag fault (page)",
            0b0111 => "Translation fault (page)",
            0b1000 => "Synchronous external abort, non-translation",
            0b1001 => "Domain fault (section)",
            0b1011 => "Domain fault (page)",
            0b1100 => "Synchronous external abort on translation table walk, \
                       1st level",
            0b1101 => "Permission fault (section)",
            0b1110 => "Synchronous external abort on translation table walk, \
                       2nd level",
            0b1111 => "Permission fault (page)",
            _ => "Unknown fault",
        }
    }
    else
    {
        match fault_type
        {
            0b0110 => "Asynchronous external abort",
            _ => "Unknown fault"
        }
    }
}

#[no_mangle]
pub extern fn prefetch_abort_handler(instr_addr: usize, status: u32)
{
    let fault_desc = fault_description(status);

    if system_control::get_spsr() & 0b11111 == ProcessorMode::User as u32
    {
        if let Some(process) = scheduler::current_process()
        {
            error!("{}: Prefetch abort at instruction {:#x}: {}.",
                   process.name, instr_addr, fault_desc);
        }
        else
        {
            panic!("Prefetch abort while no current process running");
        }
        unsafe { asm!("svc 5" :: "{r0}"(104) :: "volatile") } // Syscall exit
    }
    else
    {
        panic!("Prefetch abort at instruction {:#x}: {}.", instr_addr, fault_desc)
    }
}

#[no_mangle]
pub extern fn data_abort_handler(instr_addr: usize, data_addr: usize,
                                 status: u32)
{
    let translation_fault = status & (0b1101 | 1 << 10) == 0b0101;
    let cache = status & (1 << 13) != 0;
    let write = status & (1 << 11) != 0;

    if translation_fault && write
    {
        // If we get a fault on a stack, try to make it grow and
        // retry the instruction
        if data_addr >= kernel_map::STACK_PAGE_LIMIT.to_addr() &&
           data_addr < kernel_map::FIRST_APPLICATION_PAGE.to_addr()
        {
            kernel_map::grow_svc_stack(data_addr);
            return;
        }
        else if data_addr >= application_map::STACK_PAGE_LIMIT.to_addr()
        {
            application_map::grow_current_stack(data_addr).unwrap();
            return;
        }
    }

    let fault_desc = fault_description(status);

    if system_control::get_spsr() & 0b11111 == ProcessorMode::User as u32
    {
        // Do not panic on wrong application code
        if let Some(process) = scheduler::current_process()
        {
            error!("{}: Data abort at instruction {:#x}.\n\
                Invalid {} at {:#x}: {}.", process.name,
                instr_addr, if write { "write" } else { "read" }, data_addr,
                fault_desc);
        }
        else
        {
            panic!("Data abort while no current process running: {}", fault_desc);
        }
        unsafe { asm!("svc 5" :: "{r0}"(105) :: "volatile") } // Syscall exit
    }
    else
    {
        panic!("Data abort at instruction {:#x}.\n\
                Invalid {} at {:#x}{}: {}.",
                instr_addr, if write { "write" } else { "read" }, data_addr,
                if cache { " (cache maintenance)" } else { "" }, fault_desc);
    }
}


#[no_mangle]
pub extern fn irq_handler(reg_ctx: &mut RegisterContext)
{
    drivers::interrupts::handle_irq();
    scheduler::check_schedule(reg_ctx);
}

#[no_mangle]
pub extern fn fiq_handler()
{
    drivers::interrupts::handle_fiq();
}
