use filesystem::fat32::file::File as FatFile;
use filesystem::Dir as DirTrait;
use filesystem::DirEntry;
use filesystem::fat32::dir_entry::DirEntry as FatDirEntry;
use io;
use alloc::{Vec, boxed::Box};
use filesystem::File;

pub struct Dir<'a>
{
    pub file: FatFile<'a>
}

impl<'a> DirTrait for Dir<'a>
{
    fn list_entries(&mut self) -> Vec<DirEntry>
    {
        let mut done = false;
        let mut entries = Vec::new();
        let mut pos = 0;
        while !done
        {
            let dir_entry = FatDirEntry::dump(&mut self.file, pos);
            if dir_entry.name[0] == 0 {
                done = true
            }
            else {
                entries.push(dir_entry.to_vfs_dir_entry());
                pos = dir_entry.pos + 32;
            }
        }
        entries
    }

    fn get_file(&mut self, name: &str) -> io::Result<Box<File>>
    {
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

}
