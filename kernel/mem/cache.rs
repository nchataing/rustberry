coproc_reg!
{
    ICIALLUIS : p15, c7, 0, c1, 0;
    BPIALLIS  : p15, c7, 0, c1, 6;
    TLBIALLIS : p15, c8, 0, c3, 0;
}

pub fn invalidate_instr_cache()
{
    unsafe
    {
        ICIALLUIS::write(0);
    }
}

pub fn invalidate_branch_predictor()
{
    unsafe
    {
        BPIALLIS::write(0);
    }
}

pub fn invalidate_tlb()
{
    unsafe
    {
        TLBIALLIS::write(0);
    }
}
