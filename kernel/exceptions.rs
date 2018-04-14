use core::ptr::read_volatile;
use drivers::uart::{Uart, Write};

#[no_mangle]
pub extern fn undefined_instruction_handler(instr_addr: usize) -> !
{
    panic!("Undefined instruction at {:#x}", instr_addr)
}


#[no_mangle]
pub unsafe extern fn software_interrupt_handler(_a1: usize, _a2: usize,
                                                _a3: usize, _a4: usize,
                                                call_addr: usize)
{
    // We can get the content of lr when the interruption occur as the 5th
    // argument as it was pushed on the stack by the assembly code.

    let syscall_id = read_volatile((call_addr-4) as *const u32) & 0x00ff_ffff;
    write!(Uart, "Syscall {} at {:#x}\n", syscall_id, call_addr-4).unwrap();
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
pub extern fn prefetch_abort_handler(instr_addr: usize, status: u32) -> !
{
    let fault_desc = fault_description(status);
    panic!("Prefetch abort at instruction {:#x}: {}.", instr_addr, fault_desc)
}

#[no_mangle]
pub extern fn data_abort_handler(instr_addr: usize, data_addr: usize,
                                 status: u32) -> !
{
    let cache = status & (1 << 13) != 0;
    let write = status & (1 << 11) != 0;
    let fault_desc = fault_description(status);

    panic!("Data abort at instruction {:#x}.\n\
           Invalid {} at {:#x}{}: {}.",
           instr_addr, if write { "write" } else { "read" }, data_addr,
           if cache { " (cache maintenance)" } else { "" }, fault_desc)
}

