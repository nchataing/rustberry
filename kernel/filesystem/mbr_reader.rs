use emmc::{SdCard, BLOCK_SIZE};

const MAGIC       : u16   = 0x55AA;
const FIRST_ENTRY : usize = 0x1BE;
const ENTRY_SIZE  : usize = 0x10;

#[derive(Debug, Clone, Copy)]
pub struct Partition
{
    pub id: usize,
    pub fst_sector: u32,
    pub size: u32,
    pub bootable: bool,
    // pub part_type: u8,
}

impl Partition
{
    fn new_empty() -> Partition
    {
        Partition
        {
            id: 0,
            fst_sector: 0,
            size: 0,
            bootable: false
        }
    }
}

#[derive(Debug)]
pub enum MbrError
{
    InvalidMbr(usize),
    InvalidPartitionStatus(usize),
}

pub fn read_partition_table(card: SdCard) -> Result<[Partition; 4], MbrError>
{
    let mut mbr_block = [0; BLOCK_SIZE];
    // The Master Boot Record is on the first block of the sd card
    card.read(&mut mbr_block, 0).unwrap();
    let magic_val = mbr_block[BLOCK_SIZE-2] as u16 *0x100 +
        mbr_block[BLOCK_SIZE-1] as u16;
    if magic_val != MAGIC
    {
        return Err(MbrError::UnvalidMbr(magic_val as usize))
    }
    
    let mut parts = [Partition::new_empty(); 4];

    for part_nb in 0 .. 4
    {
        let entry_addr = FIRST_ENTRY + part_nb * ENTRY_SIZE;
        parts[part_nb].id = part_nb + 1;
        parts[part_nb].bootable = match mbr_block[entry_addr]
        {
            0x80 => true,
            0x00 => false,
            _ => return Err(MbrError::UnvalidPartitionStatus(part_nb))
        };
        // parts[part_nb].part_type = mbr_block[entry_addr + 0x4];
        parts[part_nb].fst_sector =
            (mbr_block[entry_addr + 0x8] as u32)+
            (mbr_block[entry_addr + 0x9] as u32) * 0x100 +
            (mbr_block[entry_addr + 0xa] as u32) * 0x10000 +
            (mbr_block[entry_addr + 0xb] as u32) * 0x1000000;

        parts[part_nb].size =
            (mbr_block[entry_addr + 0xc] as u32) +
            (mbr_block[entry_addr + 0xd] as u32) * 0x100 + 
            (mbr_block[entry_addr + 0xe] as u32) * 0x10000 +
            (mbr_block[entry_addr + 0xf] as u32) * 0x1000000;
    }

    Ok(parts)
}

