use filesystem::table::Fat;

pub struct File<'a>
{
    fst_cluster: Option<u32>,
    cur_cluster: Option<u32>,
    // Current position in file
    offset: u32,
    fs: &'a Fat<'a>,
}

impl <'a> File <'a>
{
    pub fn new() -> Self
    {

    }
}
