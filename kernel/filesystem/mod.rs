use alloc::{Vec, String};
use alloc::boxed::Box;
use io;
use io::{Read, Write, Seek};

pub trait File : Read + Write + Seek { }
impl<T: Read + Write + Seek> File for T { }

pub enum FileType
{
    File,
    Directory,
    Symlink,
    CharacterDevice,
    BlockDevice
}

pub struct DirEntry
{
    pub name: String,
    pub typ: FileType,
    pub size: usize,
}

impl DirEntry
{
    pub fn print(&self)
    {
        match self.typ {
            FileType::File => print!("FILE "),
            FileType::Directory => print!("DIR  "),
            _ => ()
        };
        print!("{}\n", self.name);
    }
}

pub trait Dir : 'static
{
    fn list_entries(&mut self) -> Vec<DirEntry>;

    fn get_file(&mut self, name: &str) -> io::Result<Box<File>>;
    fn get_subdir(&mut self, name: &str) -> io::Result<Box<Dir>>;
    fn add_file(&mut self, name: &str) -> io::Result<()>;
    fn add_subdir(&mut self, name: &str) -> io::Result<()>;
    fn delete_child(&mut self, name: &str) -> io::Result<()>;

    fn box_clone(&self) -> Box<Dir>;

    fn open_file(&mut self, path: &str) -> io::Result<Box<File>>
    {
        let path : Vec<&str> = path.rsplitn(2, '/').collect();
        if path.len() == 1
        {
            self.get_file(path[0])
        }
        else
        {
            let mut dir = self.open_dir(path[1])?;
            dir.get_file(path[0])
        }
    }

    fn open_dir(&mut self, path: &str) -> io::Result<Box<Dir>>
    {
        let mut current_dir : Option<Box<Dir>> = None;
        for subdir in path.split('/')
        {
            if subdir.len() == 0 { continue; }

            match current_dir
            {
                Some(mut cur_dir) =>
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

    fn create_file(&mut self, path: &str) -> io::Result<()>
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

    fn create_dir(&mut self, path: &str) -> io::Result<()>
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

    fn delete(&mut self, path: &str) -> io::Result<()>
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

impl Clone for Box<Dir>
{
    fn clone(&self) -> Box<Dir>
    {
        self.box_clone()
    }
}

pub mod mbr_reader;
pub mod virtualfs;
pub mod devfs;
pub mod buffer_io;
pub mod fat32;
