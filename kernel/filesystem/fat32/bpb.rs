use crate::filesystem::buffer_io::*;
use drivers::emmc::{SdCard, BLOCK_SIZE};

#[derive(Default, Debug)]
pub struct BiosParameterBlock {
    bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fats: u8,
    root_entries: u16,
    // = 0 if there are more than 65535 sectors
    total_sector_16: u16,
    media: u8,
    // in case of FAT32, sectors_per_fat = 0
    sectors_per_fat_16: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    // = 0 if there are less than 65535 sectors
    total_sectors_32: u32,

    // Extended BIOS Parameter Block
    pub sectors_per_fat_32: u32,
    extended_flags: u16,
    fs_version: u16,
    pub root_fst_cluster: u32,
    fs_info_sector: u16,
    backup_boot_sector: u16,
    // These bytes should be 0
    reserved_0: [u8; 12],
    drive_num: u8,
    reserved_1: u8,
    ext_sig: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    // This should be "FAT32   "
    fs_type_label: [u8; 8],
}

#[derive(Debug)]
pub enum FatError {
    FsIsNotFat32,
    BadLinking,
    NoClusterAvailable,
}

pub fn dump(card: &SdCard, fat_part_block: usize) -> Result<BiosParameterBlock, FatError> {
    let mut bpb_block = [0; BLOCK_SIZE];
    card.read(&mut bpb_block, fat_part_block).unwrap();

    let mut bpb: BiosParameterBlock = Default::default();
    bpb.bytes_per_sector = read_u16(&bpb_block, 0xb);
    bpb.sectors_per_cluster = bpb_block[0xd];
    bpb.reserved_sectors = read_u16(&bpb_block, 0xe);
    bpb.fats = bpb_block[0x10];
    bpb.root_entries = read_u16(&bpb_block, 0x11);
    bpb.total_sector_16 = read_u16(&bpb_block, 0x13);
    bpb.media = bpb_block[0x15];
    bpb.sectors_per_fat_16 = read_u16(&bpb_block, 0x16);
    bpb.sectors_per_track = read_u16(&bpb_block, 0x18);
    bpb.heads = read_u16(&bpb_block, 0x1a);
    bpb.hidden_sectors = read_u32(&bpb_block, 0x1c);
    bpb.total_sectors_32 = read_u32(&bpb_block, 0x20);

    if bpb.sectors_per_fat_16 != 0 {
        return Err(FatError::FsIsNotFat32);
    } else {
        bpb.sectors_per_fat_32 = read_u32(&bpb_block, 0x24);
        bpb.extended_flags = read_u16(&bpb_block, 0x28);
        bpb.fs_version = read_u16(&bpb_block, 0x2a);
        bpb.root_fst_cluster = read_u32(&bpb_block, 0x2c);
        bpb.fs_info_sector = read_u16(&bpb_block, 0x30);
        bpb.backup_boot_sector = read_u16(&bpb_block, 0x32);
        for i in 0..12 {
            bpb.reserved_0[i] = bpb_block[0x34 + i]
        }
        bpb.drive_num = bpb_block[0x40];
        bpb.reserved_1 = bpb_block[0x41];
        bpb.ext_sig = bpb_block[0x42];
        bpb.volume_id = read_u32(&bpb_block, 0x43);
        for i in 0..11 {
            bpb.volume_label[i] = bpb_block[0x47 + i]
        }
        for i in 0..8 {
            bpb.fs_type_label[i] = bpb_block[0x52 + i]
        }
    }

    Ok(bpb)
}
