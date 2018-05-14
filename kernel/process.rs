use alloc::String;
use memory;

#[derive(Debug)]
#[repr(C)]
pub struct RegisterContext
{
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r4: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    r12: u32,
    sp: *const u32,
    lr: *const u32,
    pc: *const u32,
    psr: u32,
}

enum ProcessState
{
    Runnable,
    BlockedWriting,
    BlockedReading,
    WaitingChildren,
    Zombie
}

pub struct Process
{
    regs: RegisterContext,
    state: ProcessState,
    name: String,
    memory_map: memory::application_map::ApplicationMap,
}
