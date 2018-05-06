use alloc::{boxed::Box, String};
use mem;

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
    sp: *mut u32,
    lr: *mut u32,
    pc: *mut u32,
    psr: u32,
}

pub struct Process
{
    regs: RegisterContext,
    pid: u32,
    name: String,
    mmu_tbl: Box<mem::mmu::SectionTable>,
}

