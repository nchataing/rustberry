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

#[no_mangle]
pub extern fn prefetch_abort_handler(instr_addr: usize) -> !
{
    panic!("Prefetch abort at {:#x}", instr_addr)
}

#[no_mangle]
pub extern fn data_abort_handler(instr_addr: usize, data_addr: usize,
                                 status: u32) -> !
{
    panic!("Data abort at {:#x} on {:#x}. Data fault status register = {:#x}.",
           instr_addr, data_addr, status)
}

