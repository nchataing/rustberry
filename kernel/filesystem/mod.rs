use alloc::{Vec, String};
use alloc::boxed::Box;
use io;
use io::{Read, Write, Seek};

trait File : Read + Write + Seek { }

enum FileType
{
    File,
    Directory,
    Symlink,
    CharacterDevice,
    BlockDevice
}

struct DirEntry
{
    name: String,
    typ: FileType,
    size: usize,
}

trait Dir
{
    fn list_entries(&self) -> Vec<DirEntry>;
    fn open_file(&self, name: &str) -> io::Result<Box<File>>;
    fn open_subdir(&self, name: &str) -> io::Result<Box<Dir>>;
    fn create_file(&mut self, name: &str) -> io::Result<()>;
    fn create_subdir(&mut self, name: &str) -> io::Result<()>;
    fn delete(&mut self, name: &str) -> io::Result<()>;
}

pub mod mbr_reader;
pub mod io;
pub mod fat32;

