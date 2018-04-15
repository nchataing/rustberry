coproc_reg!
{
    SCTLR : p15, c1, 0, c0, 0;
    CPACR : p15, c1, 0, c0, 2;

    ICIALLUIS : p15, c7, 0, c1, 0;
    BPIALLIS  : p15, c7, 0, c1, 6;
    TLBIALLIS : p15, c8, 0, c3, 0;
}

bitflags!
{
    pub struct Features : u32
    {
        const MMU = 1 << 0;
        const ALIGNMENT_CHECK = 1 << 1;
        const CACHE = 1 << 2;
        const SWP_INSTRUCTION = 1 << 10;
        const BRANCH_PREDICTION = 1 << 11;
        const INSTRUCTION_CACHE = 1 << 12;
        const TEX_REMAP = 1 << 28;
        const ACCESS_FLAG = 1 << 29;
    }
}

pub unsafe fn enable(features: Features)
{
    SCTLR::set_bits(features.bits());
}

pub unsafe fn disable(features: Features)
{
    SCTLR::reset_bits(features.bits());
}

pub fn enable_fpu()
{
    unsafe
    {
        CPACR::write(0b1111 << 20);
        asm!("vmsr FPEXC, $0" :: "r"(1 << 30) :: "volatile");
    }
}

pub fn disable_fpu()
{
    unsafe
    {
        asm!("vmsr FPEXC, $0" :: "r"(0) :: "volatile");
        CPACR::write(0);
    }
}

pub fn wipe_instr_cache()
{
    unsafe
    {
        ICIALLUIS::write(0);
    }
}

pub fn wipe_branch_predictor()
{
    unsafe
    {
        BPIALLIS::write(0);
    }
}

pub fn wipe_tlb()
{
    unsafe
    {
        TLBIALLIS::write(0);
    }
}
