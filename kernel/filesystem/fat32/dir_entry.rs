use crate::filesystem::buffer_io::*;
use crate::filesystem::fat32::file::File;
use crate::filesystem::DirEntry as VfsDirEntry;
use crate::filesystem::FileType;
use io::*;

use alloc::string::*;

// File attributes
const READ_ONLY: u8 = 0x01;
const HIDDEN: u8 = 0x02;
const SYSTEM: u8 = 0x04;
const VOLUME_ID: u8 = 0x08;
const DIRECTORY: u8 = 0x10;
const ARCHIVE: u8 = 0x20;
// long name descriptor
const LND: u8 = READ_ONLY | HIDDEN | SYSTEM | VOLUME_ID;

#[derive(Default, Debug, Clone)]
pub struct DirEntry {
    // General information
    pub name: [u8; 11],
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
    pub pos: usize,
    dirty: bool,

    // long name option
    long_name: Option<String>,
    long_descr_pos: usize,
}

pub enum Typ {
    Some(DirEntry),
    Unused,
    None,
}

impl DirEntry {
    pub fn dump(file: &mut File, pos: usize) -> Typ {
        let mut buf = [0; 32];
        let mut ext_descr = true;
        match file.seek(SeekFrom::Start(pos as u64)) {
            Ok(_) => (),
            Err(_) => return Typ::None,
        }
        file.read_exact(&mut buf).unwrap();

        if buf[0] == 0x0 {
            return Typ::None;
        }
        if buf[0] == 0xE5 {
            return Typ::Unused;
        }

        let mut long_name: String = "".to_string();
        let mut descr_pos = pos;

        while ext_descr {
            if buf[11] == LND {
                let mut utf16_buf: [u16; 13] = [0; 13];
                utf16_buf[0] = read_u16(&buf, 1);
                utf16_buf[1] = read_u16(&buf, 3);
                utf16_buf[2] = read_u16(&buf, 5);
                utf16_buf[3] = read_u16(&buf, 7);
                utf16_buf[4] = read_u16(&buf, 9);
                utf16_buf[5] = read_u16(&buf, 14);
                utf16_buf[6] = read_u16(&buf, 16);
                utf16_buf[7] = read_u16(&buf, 18);
                utf16_buf[8] = read_u16(&buf, 20);
                utf16_buf[9] = read_u16(&buf, 22);
                utf16_buf[10] = read_u16(&buf, 24);
                utf16_buf[11] = read_u16(&buf, 28);
                utf16_buf[12] = read_u16(&buf, 30);
                let long_name_part = String::from_utf16(&utf16_buf).unwrap();
                long_name.insert_str(0, &long_name_part);
                descr_pos += 32;
                file.read_exact(&mut buf).unwrap();
            } else {
                ext_descr = false
            }
        }

        let mut entry: DirEntry = Default::default();
        // dump
        entry.name.clone_from_slice(&buf[0..11]);
        entry.attrs = buf[11];
        entry.reserved_0 = buf[12];
        entry.create_time_0 = buf[13];
        entry.create_time_1 = read_u16(&buf, 14);
        entry.create_date = read_u16(&buf, 16);
        entry.access_date = read_u16(&buf, 18);
        entry.fst_cluster_hi = read_u16(&buf, 20);
        entry.modify_time = read_u16(&buf, 22);
        entry.modify_date = read_u16(&buf, 24);
        entry.fst_cluster_lo = read_u16(&buf, 26);
        entry.size = read_u32(&buf, 28);
        // other
        entry.pos = descr_pos;
        entry.dirty = false;

        if long_name.len() == 0 {
            entry.long_name = None;
            entry.long_descr_pos = 0;
        } else {
            entry.long_name = Some(long_name);
            entry.long_descr_pos = pos;
        }

        Typ::Some(entry)
    }

    pub fn is_dir(&self) -> bool {
        self.attrs & DIRECTORY == DIRECTORY
    }

    pub fn print(&self) -> () {
        if self.is_dir() {
            print!("DIR  ");
        } else {
            print!("FILE ");
        }
        if let Some(ref name) = self.long_name {
            println!("{}", name);
        } else {
            println!("{}", String::from_utf8(self.name.to_vec()).unwrap());
        }
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn fst_cluster(&self) -> u32 {
        ((self.fst_cluster_hi as u32) << 16) & self.fst_cluster_lo as u32
    }

    pub fn set_fst_cluster(&mut self, cluster: u32) {
        self.fst_cluster_hi = (cluster >> 16) as u16;
        self.fst_cluster_lo = cluster as u16;
    }

    pub fn to_vfs_dir_entry(&self) -> VfsDirEntry {
        let name = if let Some(ref name) = self.long_name {
            name.clone()
        } else {
            String::from_utf8(self.name.to_vec()).unwrap()
        };
        let typ = if self.is_dir() {
            FileType::Directory
        } else {
            FileType::File
        };
        let size = self.size as usize;
        VfsDirEntry { name, typ, size }
    }

    pub fn get_name(&self) -> String {
        if let Some(ref name) = self.long_name {
            name.clone()
        } else {
            String::from_utf8(self.name.to_vec()).unwrap()
        }
    }
}
