coproc_reg!
{
    ICIALLUIS : p15, c7, 0, c1, 0;
    BPIALLIS  : p15, c7, 0, c1, 6;
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

/**
 * This module contains cache maintenance operations on the
 * Translation Lookaside Buffer.
 * It must be used after changes in the translation table.
 */
pub mod tlb
{
    use mem::PageId;

    coproc_reg!
    {
        TLBIALLIS  : p15, c8, 0, c3, 0;
        TLBIMVAIS  : p15, c8, 0, c3, 1;
        TLBIASIDIS : p15, c8, 0, c3, 2;
        TLBIMVAAIS : p15, c8, 0, c3, 3;
    }

    pub fn invalidate_all()
    {
        unsafe
        {
            TLBIALLIS::write(0);
        }
    }

    pub fn invalidate_page(vaddr_base: PageId)
    {
        unsafe
        {
            TLBIMVAAIS::write(vaddr_base.to_addr() as u32);
        }
    }

    pub fn invalidate_asid(asid: u8)
    {
        unsafe
        {
            TLBIASIDIS::write(asid as u32);
        }
    }

    pub fn invalidate_asid_page(asid: u8, vaddr_base: PageId)
    {
        unsafe
        {
            TLBIMVAIS::write(asid as u32 | vaddr_base.to_addr() as u32);
        }
    }
}

