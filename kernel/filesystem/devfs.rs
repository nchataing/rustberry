use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::{BTreeMap, Vec};
use core::cmp::min;
use drivers::CharacterDevice;
use filesystem::{Dir, DirEntry, File, FileType};
use io;
use io::{Read, Seek, Write};

struct DeviceFile {
    device: Rc<CharacterDevice>,
}

impl Read for DeviceFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_size = min(buf.len(), 256);
        for i in 0..read_size {
            buf[i] = self.device.read_byte();
        }
        Ok(read_size)
    }
}

impl Write for DeviceFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let write_size = min(buf.len(), 256);
        for i in 0..write_size {
            self.device.write_byte(buf[i]);
        }
        Ok(write_size)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.device.flush();
        Ok(())
    }
}

impl Seek for DeviceFile {
    fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
        Err(io::Error {
            kind: io::ErrorKind::InvalidInput,
            error: "cannot seek inside device",
        })
    }
}

#[derive(Clone)]
pub struct DeviceDir {
    devices: BTreeMap<String, Rc<CharacterDevice>>,
}

impl DeviceDir {
    pub fn new() -> DeviceDir {
        DeviceDir {
            devices: BTreeMap::new(),
        }
    }

    pub fn add_device(&mut self, name: String, dev: Rc<CharacterDevice>) {
        self.devices.insert(name, dev);
    }

    pub fn remove_device(&mut self, name: &str) {
        self.devices.remove(name);
    }
}

impl Dir for DeviceDir {
    fn list_entries(&mut self) -> Vec<DirEntry> {
        self.devices
            .keys()
            .map(|x| DirEntry {
                name: x.clone(),
                typ: FileType::CharacterDevice,
                size: 0,
            })
            .collect()
    }

    fn get_file(&mut self, name: &str) -> io::Result<Box<File>> {
        match self.devices.get(name) {
            None => Err(io::Error {
                kind: io::ErrorKind::NotFound,
                error: "device not found",
            }),
            Some(dev) => Ok(Box::new(DeviceFile {
                device: dev.clone(),
            })),
        }
    }

    fn get_subdir(&mut self, _: &str) -> io::Result<Box<Dir>> {
        Err(io::Error {
            kind: io::ErrorKind::NotFound,
            error: "no subdirectories in devfs",
        })
    }

    fn add_file(&mut self, _: &str) -> io::Result<()> {
        Err(io::Error {
            kind: io::ErrorKind::InvalidInput,
            error: "devfs is read only",
        })
    }

    fn add_subdir(&mut self, _: &str) -> io::Result<()> {
        Err(io::Error {
            kind: io::ErrorKind::InvalidInput,
            error: "devfs is read only",
        })
    }

    fn delete_child(&mut self, _: &str) -> io::Result<()> {
        Err(io::Error {
            kind: io::ErrorKind::InvalidInput,
            error: "devfs is read only",
        })
    }

    fn box_clone(&self) -> Box<Dir> {
        Box::new(self.clone())
    }
}
