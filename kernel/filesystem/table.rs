use emmc::{SdCard, BLOCK_SIZE};
use filesystem::fs::{FatError, BiosParameterBlock};
use filesystem::io::*;

pub enum Entry
{
    Free,
    Full(u32),
    Bad,
    EndOfChain,
}

pub struct Fat<'a>
{
    // fst sector of the FAT, regardless of the partition
    fst_sector: usize,
    sectors_per_cluster: usize,
    buffer: [u8; 512],
    card: &'a SdCard,
}

impl<'a> Fat<'a>
{
    // which indicates which FAT is being used.
    pub fn new(bpb: &BiosParameterBlock, fst_part_sector: usize, which: usize,
               card: &'a SdCard) 
        -> Fat<'a>
    {
        let fst_sector = fst_part_sector + bpb.reserved_sectors as usize
                        + which * (bpb.sectors_per_fat_32 as usize);
        let sectors_per_cluster = bpb.sectors_per_cluster as usize;
        Fat { fst_sector, buffer : [0; 512], card, sectors_per_cluster }
    }

    pub fn get_entry(&mut self, cluster: u32) -> Result<Entry, FatError>
    {
        let fat_offset = cluster as usize * 4;
        let fat_sector = self.fst_sector + (fat_offset / BLOCK_SIZE);
        let entry_offset = fat_offset % BLOCK_SIZE;
        self.card.read(&mut self.buffer, fat_sector).unwrap();
        match read_u32(&self.buffer, entry_offset) & 0x0FFF_FFFF
        {
            0x0 => Ok(Entry::Free),
            0xFFF_FFF7 => Ok(Entry::Bad),
            0xFFF_FFF8 ... 0xFFF_FFFF => Ok(Entry::EndOfChain),
            n => Ok(Entry::Full(n))
        }
    }

    pub fn set_entry(&mut self, cluster: u32, entry: Entry) 
    {
        let fat_offset = cluster as usize * 4;
        let fat_sector = self.fst_sector + (fat_offset / BLOCK_SIZE);
        let entry_offset = fat_offset % BLOCK_SIZE;
        self.card.read(&mut self.buffer, fat_sector).unwrap();
        let coded_entry = match entry 
        {
            Entry::Free => 0x0,
            Entry::Full(n) => n & 0x0FFF_FFFF,
            Entry::Bad => 0xFFF_FFF7,
            Entry::EndOfChain => 0xFFF_FFFF
        };
        write_u32(&mut self.buffer, entry_offset, coded_entry);
        self.card.write(&self.buffer, fat_sector).unwrap();
    }
}
