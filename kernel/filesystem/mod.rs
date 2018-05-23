use alloc::{Vec, String};
use alloc::boxed::Box;
use io;
use io::{Read, Write, Seek};

trait File : Read + Write + Seek { }
impl<T: Read + Write + Seek> File for T { }

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

    fn get_file(&self, name: &str) -> io::Result<Box<File>>;
    fn get_subdir(&self, name: &str) -> io::Result<Box<Dir>>;
    fn add_file(&mut self, name: &str) -> io::Result<()>;
    fn add_subdir(&mut self, name: &str) -> io::Result<()>;
    fn delete_child(&mut self, name: &str) -> io::Result<()>;

    fn open_file(&self, path: &str) -> io::Result<Box<File>> where Self: Sized
    {
        let path : Vec<&str> = path.rsplitn(2, '/').collect();
        if path.len() == 1
        {
            self.get_file(path[0])
        }
        else
        {
            let dir = self.open_dir(path[1])?;
            dir.get_file(path[0])
        }
    }

    fn open_dir(&self, path: &str) -> io::Result<Box<Dir>> where Self: Sized
    {
        let mut current_dir : Option<Box<Dir>> = None;
        for subdir in path.split('/')
        {
            if subdir.len() == 0 { continue; }

            match current_dir
            {
                Some(cur_dir) =>
                {
                    let next_dir = cur_dir.get_subdir(subdir)?;
                    current_dir = Some(next_dir);
                }
                None =>
                {
                    current_dir = Some(self.get_subdir(subdir)?);
                }
            }
        }
        current_dir.ok_or(io::Error { kind: io::ErrorKind::InvalidInput,
                                      error: "invalid path in open_dir" })
    }

    fn create_file(&mut self, path: &str) -> io::Result<()> where Self: Sized
    {
        let path : Vec<&str> = path.rsplitn(2, '/').collect();
        if path.len() == 1
        {
            self.add_file(path[0])
        }
        else
        {
            let mut dir = self.open_dir(path[1])?;
            dir.add_file(path[0])
        }
    }

    fn create_dir(&mut self, path: &str) -> io::Result<()> where Self: Sized
    {
        let path : Vec<&str> = path.rsplitn(2, '/').collect();
        if path.len() == 1
        {
            self.add_subdir(path[0])
        }
        else
        {
            let mut dir = self.open_dir(path[1])?;
            dir.add_subdir(path[0])
        }
    }

    fn delete(&mut self, path: &str) -> io::Result<()> where Self: Sized
    {
        let path : Vec<&str> = path.rsplitn(2, '/').collect();
        if path.len() == 1
        {
            self.delete_child(path[0])
        }
        else
        {
            let mut dir = self.open_dir(path[1])?;
            dir.delete_child(path[0])
        }
    }
}

pub mod mbr_reader;
pub mod buffer_io;
pub mod fat32;

