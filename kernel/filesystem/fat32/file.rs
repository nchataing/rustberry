use crate::filesystem::fat32::dir_entry::DirEntry;
use crate::filesystem::fat32::table::{Entry, Fat};
use alloc::rc::Rc;
use core::cell::RefCell;
use core::cmp::min;
use io::*;

#[derive(Clone)]
pub struct File {
    fst_cluster: Option<u32>,
    cur_cluster: Option<u32>,
    // Current position in file
    pub offset: usize,
    entry: Option<Rc<RefCell<DirEntry>>>,
    fs: Rc<Fat>,
}

impl File {
    pub fn new(fat: Rc<Fat>, fst_cluster: Option<u32>) -> Self {
        File {
            fst_cluster,
            cur_cluster: None,
            offset: 0,
            entry: None,
            fs: fat,
        }
    }

    pub fn new_from_entry(fat: Rc<Fat>, entry: DirEntry) -> Self {
        File {
            fst_cluster: Some(entry.fst_cluster()),
            cur_cluster: None,
            offset: 0,
            entry: Some(Rc::new(RefCell::new(entry))),
            fs: fat,
        }
    }

    pub fn get_size(&self) -> Option<usize> {
        match self.entry {
            None => None,
            Some(ref e) => Some(e.borrow().size()),
        }
    }

    pub fn update_size(&mut self) {
        ()
    }

    pub fn set_fst_cluster(&mut self, cluster: u32) {
        self.fst_cluster = Some(cluster);
        match self.entry {
            Some(ref mut e) => e.borrow_mut().set_fst_cluster(cluster),
            None => {}
        }
    }

    pub fn bytes_left_in_file(&self) -> Option<usize> {
        match self.entry {
            None => None,
            Some(ref ent) => Some(ent.borrow().size() as usize - self.offset),
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let cluster_size = self.fs.cluster_size;
        let cur_cluster = if self.offset == 0 {
            self.fst_cluster
        } else if self.offset % cluster_size == 0 {
            // Get next cluster
            match self.cur_cluster {
                None => self.fst_cluster,
                Some(n) => match self.fs.get_entry(n).unwrap() {
                    Entry::Free | Entry::Bad => {
                        let error = Error {
                            kind: ErrorKind::InvalidData,
                            error: "Bad next cluster in file",
                        };
                        return Err(error);
                    }
                    Entry::EndOfChain => return Ok(0),
                    Entry::Full(m) => Some(m),
                },
            }
        } else {
            self.cur_cluster
        };

        let cur_cluster = match cur_cluster {
            // EOF is reached
            None => return Ok(0),
            Some(n) => n,
        };

        let offset_in_cluster = self.offset % cluster_size;
        let bytes_left_in_cluster = cluster_size - offset_in_cluster;
        let bytes_left_in_file = self.bytes_left_in_file().unwrap_or(bytes_left_in_cluster);
        let read_size = min(min(buf.len(), bytes_left_in_cluster), bytes_left_in_file);
        if read_size == 0 {
            // EOF is reached
            return Ok(0);
        }

        let mut cluster_buf = vec![0; cluster_size];
        self.fs.read_cluster(&mut cluster_buf, cur_cluster);
        buf[0..read_size]
            .clone_from_slice(&cluster_buf[offset_in_cluster..offset_in_cluster + read_size]);
        // Update file info
        self.offset += read_size;
        self.cur_cluster = Some(cur_cluster);

        Ok(read_size)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let cluster_size = self.fs.cluster_size;
        let offset = self.offset % cluster_size;
        let bytes_left_in_cluster = cluster_size - offset;
        let write_size = min(buf.len(), bytes_left_in_cluster);

        if write_size == 0 {
            return Ok(0);
        }

        let cur_cluster = if offset == 0 {
            // get next cluster
            let next_cluster = match self.cur_cluster {
                None => self.fst_cluster,
                Some(n) => match self.fs.get_entry(n).unwrap() {
                    Entry::Free | Entry::Bad => {
                        let error = Error {
                            kind: ErrorKind::InvalidData,
                            error: "Bad next cluster in file",
                        };
                        return Err(error);
                    }
                    Entry::EndOfChain => None,
                    Entry::Full(m) => Some(m),
                },
            };

            match next_cluster {
                Some(n) => n,
                // A new cluster should be allocated
                None => {
                    let new_cluster = self.fs.alloc_cluster(self.cur_cluster).unwrap();
                    if self.fst_cluster.is_none() {
                        self.set_fst_cluster(new_cluster);
                    }
                    // TODO: zero new directory cluster
                    new_cluster
                }
            }
        } else {
            match self.cur_cluster {
                Some(n) => n,
                // TODO: allocate cluster instead of panicking
                None => panic!("Offset inside cluster but no cluster allocated"),
            }
        };

        let mut cluster_buf = vec![0; cluster_size];
        self.fs.read_cluster(&mut cluster_buf, cur_cluster);
        let write_slice = &mut cluster_buf[offset..offset + write_size];
        write_slice.clone_from_slice(&buf[0..write_size]);

        self.offset += write_size;
        self.cur_cluster = Some(cur_cluster);
        self.update_size();
        Ok(write_size)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (cur_pos, offset) = match pos {
            SeekFrom::Start(s) => (0, s as usize),
            SeekFrom::End(neg_offset) => match self.get_size() {
                None => panic!("Unknown file size"),
                Some(size) => {
                    let pos = size as i64 + neg_offset;
                    if neg_offset >= 0 {
                        return Ok(size as u64);
                    } else if pos <= 0 {
                        let error = Error {
                            kind: ErrorKind::InvalidInput,
                            error: "Can't read before byte 0 of file",
                        };
                        return Err(error);
                    } else if pos as usize >= self.offset {
                        (self.offset, pos as usize - self.offset)
                    } else {
                        (0, (size as i64 + neg_offset) as usize)
                    }
                }
            },
            SeekFrom::Current(offset) => {
                if offset >= 0 {
                    (self.offset, offset as usize)
                } else if self.offset as i64 - offset <= 0 {
                    let error = Error {
                        kind: ErrorKind::InvalidInput,
                        error: "Can't read before byte 0 of file",
                    };
                    return Err(error);
                } else {
                    (0, self.offset - (offset as usize))
                }
            }
        };

        if cur_pos == 0 {
            self.cur_cluster = self.fst_cluster;
        }
        let cluster_size = self.fs.cluster_size;
        let nb_cluster_to_pass = cur_pos / cluster_size - (cur_pos + offset) / cluster_size;

        // Get to good cluster
        for _i in 0..nb_cluster_to_pass {
            self.cur_cluster = match self.cur_cluster {
                None => {
                    let error = Error {
                        kind: ErrorKind::InvalidData,
                        error: "End of cluster chain reached",
                    };
                    return Err(error);
                }
                Some(n) => match self.fs.get_entry(n).unwrap() {
                    Entry::Free | Entry::Bad => {
                        let error = Error {
                            kind: ErrorKind::InvalidData,
                            error: "Bad next cluster in file",
                        };
                        return Err(error);
                    }
                    Entry::EndOfChain => None,
                    Entry::Full(m) => Some(m),
                },
            }
        }
        self.offset = offset - cur_pos;
        Ok(self.offset as u64)
    }
}
