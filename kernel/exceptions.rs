use drivers;
use memory::{kernel_map, application_map};
use process::RegisterContext;
use scheduler;

#[no_mangle]
pub extern fn undefined_instruction_handler(instr_addr: usize) -> !
{
    panic!("Undefined instruction at {:#x}", instr_addr)
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
    panic!("Prefetch abort at instruction {:#x}: {}.", instr_addr, fault_desc)
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

    // TODO: Do not panic on wrong application code
    let fault_desc = fault_description(status);
    panic!("Data abort at instruction {:#x}.\n\
           Invalid {} at {:#x}{}: {}.",
           instr_addr, if write { "write" } else { "read" }, data_addr,
           if cache { " (cache maintenance)" } else { "" }, fault_desc)
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
