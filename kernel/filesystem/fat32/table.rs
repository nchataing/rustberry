use emmc::{SdCard, BLOCK_SIZE};
use filesystem::fat32::bpb;
use filesystem::fat32::bpb::FatError;
use filesystem::buffer_io::*;
use filesystem::fat32::{file::File, dir::Dir};

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
    pub fst_sector: usize,
    pub fst_data_sector: usize,
    pub fat_size: usize,
    pub sectors_per_cluster: usize,
    pub cluster_size: usize,
    pub root_fst_cluster: u32,
    pub card: &'a SdCard,
}

impl<'a> Fat<'a>
{
    // which indicates which FAT is being used.
    pub fn new(card: &'a SdCard, fst_part_sector: usize, which: usize) 
        -> Result<Fat<'a>, FatError>
    {
        let bpb = match bpb::dump(card, fst_part_sector) {
            Ok(bpb) => bpb,
            Err(err) => return Err(err)
        };
        let fst_sector = fst_part_sector + bpb.reserved_sectors as usize
                        + which * (bpb.sectors_per_fat_32 as usize);
        let sectors_per_cluster = bpb.sectors_per_cluster as usize;
        let cluster_size = sectors_per_cluster * BLOCK_SIZE;
        let fst_data_sector = (bpb.reserved_sectors as usize) +
            (bpb.fats as usize) * (bpb.sectors_per_fat_32 as usize) ;
        Ok(Fat 
        { 
            fst_sector,
            fst_data_sector,
            card,
            fat_size: bpb.sectors_per_fat_32 as usize,
            sectors_per_cluster,
            root_fst_cluster: bpb.root_fst_cluster as u32,
            cluster_size
        })
    }

    pub fn get_entry(&self, cluster: u32) -> Result<Entry, FatError>
    {
        let mut buf = [0; 512];
        let fat_offset = cluster as usize * 4;
        let fat_sector = self.fst_sector + (fat_offset / BLOCK_SIZE);
        let entry_offset = fat_offset % BLOCK_SIZE;
        self.card.read(&mut buf, fat_sector).unwrap();
        match read_u32(&buf, entry_offset) & 0x0FFF_FFFF
        {
            0x0 => Ok(Entry::Free),
            0xFFF_FFF7 => Ok(Entry::Bad),
            0xFFF_FFF8 ... 0xFFF_FFFF => Ok(Entry::EndOfChain),
            n => Ok(Entry::Full(n))
        }
    }

    pub fn set_entry(&self, cluster: u32, entry: Entry) 
    {
        let mut buf = [0;512];
        let fat_offset = cluster as usize * 4;
        let fat_sector = self.fst_sector + (fat_offset / BLOCK_SIZE);
        let entry_offset = fat_offset % BLOCK_SIZE;
        self.card.read(&mut buf, fat_sector).unwrap();
        let coded_entry = match entry 
        {
            Entry::Free => 0x0,
            Entry::Full(n) => n & 0x0FFF_FFFF,
            Entry::Bad => 0xFFF_FFF7,
            Entry::EndOfChain => 0xFFF_FFFF
        };
        write_u32(&mut buf, entry_offset, coded_entry);
        self.card.write(&buf, fat_sector).unwrap();
    }

    pub fn next_free_cluster(&self) -> Option<u32>
    {
        let mut buf = [0;512];
        for sector in 0 .. self.fat_size
        {
            self.card.read(&mut buf, self.fst_sector + sector).unwrap();
            for offset in 0 .. BLOCK_SIZE / 4 
            {
                if read_u32(&buf, offset) == 0
                {
                    let fat_offset = (sector - self.fst_sector) * BLOCK_SIZE 
                                    + offset;
                    return Some(fat_offset as u32 / 4)
                }
            }
        }
        None
    }

    pub fn alloc_cluster(&self, cluster_from: Option<u32>) 
        -> Result<u32, FatError>
    {
        match self.next_free_cluster()
        {
            None => Err(FatError::NoClusterAvailable),
            Some(cluster) =>
            {
                if let Some(from) = cluster_from
                {
                    if let Err(e) = self.link(from, cluster)
                    {
                        return Err(e)
                    }
                }
                Ok(cluster)
            }
        }
    }

    pub fn link(&self, cluster_from: u32, cluster_to: u32)
        -> Result<(), FatError>
    {
        match self.get_entry(cluster_from).unwrap()
        {
            Entry::Free | Entry::Full(_) | Entry::Bad 
                => Err(FatError::BadLinking),
            Entry::EndOfChain => match self.get_entry(cluster_to).unwrap()
            {
                Entry::Full(_) | Entry::Bad | Entry::EndOfChain 
                    => Err(FatError::BadLinking),
                Entry::Free =>
                {
                    self.set_entry(cluster_from, Entry::Full(cluster_to));
                    self.set_entry(cluster_from, Entry::EndOfChain);
                    Ok(())
                }
            }
        }
    }

    /*pub fn get_sector(&self, mut buf: &mut [u8], 
                      cluster: u32, offset: usize) 
    {
        let sector = self.fst_sector + 
                     (cluster as usize) * self.sectors_per_cluster +
                     offset / BLOCK_SIZE;
        self.card.read(&mut buf, sector).unwrap();
    }*/

    pub fn read_cluster(&self, buf: &mut [u8], cluster: u32)
    {
        let sector = self.fst_data_sector + 
            (cluster as usize) * self.sectors_per_cluster;
        self.card.read(buf, sector).unwrap() 
    }

    pub fn write_cluster(&self, buf: &mut [u8], cluster: u32)
    {
        let sector = self.fst_data_sector + 
            (cluster as usize) * self.sectors_per_cluster;
        self.card.write(buf, sector).unwrap()
    }
    
    pub fn root_dir(&'a self) -> Dir<'a>
    {
        let root_file = File::new(self, Some(self.root_fst_cluster));
        Dir { file: root_file }
    }
}
