use filesystem::fat32::file::File as FatFile;
use filesystem::Dir as DirTrait;
use filesystem::{DirEntry};
use filesystem::fat32::dir_entry::{DirEntry as FatDirEntry, Typ};
use io;
use alloc::{Vec, boxed::Box, string::ToString};
use filesystem::File;

#[derive(Clone)]
pub struct Dir
{
    pub file: FatFile
}

impl DirTrait for Dir
{
    fn list_entries(&mut self) -> Vec<DirEntry>
    {
        let mut done = false;
        let mut entries = Vec::new();
        let mut pos = 0;
        while !done
        {
            match FatDirEntry::dump(&mut self.file, pos) {
                Typ::Some(dir_entry) => {
                    entries.push(dir_entry.to_vfs_dir_entry());
                    pos = dir_entry.pos + 32;
                },
                Typ::Unused => pos += 32,
                Typ::None => done = true 
            }
        }
        entries
    }

    fn get_file(&mut self, name: &str) -> io::Result<Box<File>>
    {
        /*let mut pos = 0;
        loop {
            if let Some(dir_entry) = FatDirEntry::dump(&mut self.file, pos)
            {
                if dir_entry.name[0] == 0 {
                    return Err(io::Error { 
                        kind : io::ErrorKind::NotFound, 
                        error : "File not found"
                    })
                }
                else if dir_entry.get_name() == name.to_string() {
                    return Ok(Box::new(
                            FatFile::new_from_entry(&self.file.fs, &dir_entry)))
                }
            }
            else {
                return Err(io::Error {
                    kind: io::ErrorKind::NotFound,
                    error: "File not found"
                    })
            }
        }*/
        panic!("")
    }
    fn get_subdir(&mut self, name: &str) -> io::Result<Box<DirTrait>>
    {
        panic!("")
    }
    fn add_file(&mut self, name: &str) -> io::Result<()>
    {
        panic!("")
    }
    fn add_subdir(&mut self, name: &str) -> io::Result<()>
    {
        panic!("")
    }
    fn delete_child(&mut self, name: &str) -> io::Result<()>
    {
        panic!("")
    }

    fn box_clone(&self) -> Box<DirTrait>
    {
        Box::new(self.clone())
    }
}
