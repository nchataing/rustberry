
// File attributes
const READ_ONLY : u8 = 0x01;
const HIDDEN    : u8 = 0x02;
const SYSTEM    : u8 = 0x04;
const VOLUME_ID : u8 = 0x08;
const DIRECTORY : u8 = 0x10;
const ARCHIVE   : u8 = 0x20;

pub struct DirEntry {
    name: [u8; 11],
    attrs: u8,
    reserved_0: u8,
    create_time_0: u8,
    create_time_1: u16,
    create_date: u16,
    access_date: u16,
    first_cluster_hi: u16,
    modify_time: u16,
    modify_date: u16,
    first_cluster_lo: u16,
    size: u32,
}

