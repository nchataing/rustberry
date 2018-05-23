//use emmc::{SdCard, BLOCK_SIZE};
use filesystem::table::Fat;

// File attributes
const READ_ONLY : u8 = 0x01;
const HIDDEN    : u8 = 0x02;
const SYSTEM    : u8 = 0x04;
const VOLUME_ID : u8 = 0x08;
const DIRECTORY : u8 = 0x10;
const ARCHIVE   : u8 = 0x20;

#[derive(Default)]
pub struct DirEntry {
    // General information
    name: [u8; 11],
    attrs: u8,
    reserved_0: u8,
    create_time_0: u8,
    create_time_1: u16,
    create_date: u16,
    access_date: u16,
    fst_cluster_hi: u16,
    modify_time: u16,
    modify_date: u16,
    fst_cluster_lo: u16,
    size: u32,

    // Position on SdCard and dirty information
    entry_sector: u32,
    entry_offset: u32,
    dirty: bool,
}

impl DirEntry 
{
    /*pub fn dump_from_file(fat: &Fat, entry_cluster: u32, entry_offset: u32) -> Self
    {
        unimplemented!()
    }*/

    pub fn size(&self) -> usize
    {
        self.size as usize
    }

    pub fn fst_cluster(&self) -> u32
    {
        ((self.fst_cluster_hi as u32) << 16) & self.fst_cluster_lo as u32
    }

    pub fn set_fst_cluster(&mut self, cluster: u32)
    {
        self.fst_cluster_hi = (cluster >> 16) as u16;
        self.fst_cluster_lo = cluster as u16;
    }
}
