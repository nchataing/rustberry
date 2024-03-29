use crate::filesystem::{Dir, DirEntry, File, FileType};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use core::cell::RefCell;
use io;

#[derive(Clone)]
pub struct VirtualDir {
    filesystem: Option<Box<Dir>>,
    children: BTreeMap<String, Rc<RefCell<VirtualDir>>>,
}

impl VirtualDir {
    pub fn new() -> VirtualDir {
        VirtualDir {
            filesystem: None,
            children: BTreeMap::new(),
        }
    }

    pub fn mount(&mut self, fs: Box<Dir>, path: &str) {
        // Add a new concrete filesystem in the virtual filesystem
        if path.len() == 0 {
            self.filesystem = Some(fs);
        } else {
            let mut path: Vec<&str> = path.splitn(2, '/').collect();
            if path.len() == 1 {
                path.push("");
            }

            if path[0].len() == 0 {
                self.mount(fs, path[1]);
            } else {
                let sub_fs = self
                    .children
                    .entry(path[0].to_string())
                    .or_insert(Rc::new(RefCell::new(VirtualDir::new())));
                sub_fs.borrow_mut().mount(fs, path[1]);
            }
        }
    }

    /*pub fn unmount(&mut self, path: &str)
    {
        // TODO
    }*/

    /// Returns the real path from the current virtual filesystem
    pub fn real_path<'p>(&self, path: &'p str) -> Option<(Box<Dir>, &'p str)> {
        if path.len() == 0 {
            return None;
        }

        let mut path: Vec<&str> = path.splitn(2, '/').collect();
        if path.len() == 1 {
            path.push("");
        }

        if path[0].len() == 0 {
            self.real_path(path[1])
        } else {
            let sub_fs = self.children.get(path[0])?.borrow();
            sub_fs
                .real_path(path[1])
                .or(self.filesystem.as_ref().map(|x| (x.box_clone(), path[1])))
        }
    }
}

impl Dir for VirtualDir {
    fn list_entries(&mut self) -> Vec<DirEntry> {
        let mut real_entries = match self.filesystem {
            None => Vec::new(),
            Some(ref mut filesystem) => filesystem.list_entries(),
        };
        let mut virtual_entries: Vec<_> = self
            .children
            .keys()
            .map(|x| DirEntry {
                name: x.clone(),
                typ: FileType::Directory,
                size: 0,
            })
            .collect();
        real_entries.append(&mut virtual_entries);
        real_entries
    }

    fn get_file(&mut self, name: &str) -> io::Result<Box<File>> {
        match self.filesystem {
            None => {
                info!("{}", name);
                Err(io::Error {
                    kind: io::ErrorKind::NotFound,
                    error: "get_file: filesystem not found",
                })
            }
            Some(ref mut fs) => fs.get_file(name),
        }
    }

    fn get_subdir(&mut self, name: &str) -> io::Result<Box<Dir>> {
        match self.children.get(name) {
            Some(vsubdir) => Ok(Box::new(vsubdir.borrow().clone())),
            None => match self.filesystem {
                Some(ref mut fs) => fs.get_subdir(name),
                None => Err(io::Error {
                    kind: io::ErrorKind::NotFound,
                    error: "get_subdir: filesystem not found",
                }),
            },
        }
    }

    fn add_file(&mut self, name: &str) -> io::Result<()> {
        match self.filesystem {
            None => Err(io::Error {
                kind: io::ErrorKind::NotFound,
                error: "filesystem not found",
            }),
            Some(ref mut fs) => fs.add_file(name),
        }
    }

    fn add_subdir(&mut self, name: &str) -> io::Result<()> {
        match self.filesystem {
            None => Err(io::Error {
                kind: io::ErrorKind::NotFound,
                error: "filesystem not found",
            }),
            Some(ref mut fs) => fs.add_subdir(name),
        }
    }

    fn delete_child(&mut self, name: &str) -> io::Result<()> {
        match self.filesystem {
            None => Err(io::Error {
                kind: io::ErrorKind::NotFound,
                error: "filesystem not found",
            }),
            Some(ref mut fs) => fs.delete_child(name),
        }
    }

    fn box_clone(&self) -> Box<Dir> {
        Box::new(self.clone())
    }
}

static mut VIRTUAL_ROOT: Option<VirtualDir> = None;

pub fn init() {
    unsafe { VIRTUAL_ROOT = Some(VirtualDir::new()) }
}

pub fn get_root() -> &'static mut VirtualDir {
    unsafe { VIRTUAL_ROOT.as_mut().unwrap() }
}
