use filesystem::fat32::table::{Fat, Entry};
use filesystem::fat32::dir_entry::DirEntry;
use core::cmp::min;
pub enum FileError
{
    Error
}

pub struct File<'a>
{
    fst_cluster: Option<u32>,
    cur_cluster: Option<u32>,
    // Current position in file
    pub offset: usize,
    entry: Option<DirEntry>,
    fs: &'a Fat<'a>,
}

impl <'a> File <'a>
{
    pub fn new(fat: &'a Fat<'a>, fst_cluster: Option<u32>) -> Self
    {
        File
        {
            fst_cluster,
            cur_cluster: fst_cluster,
            offset: 0,
            entry: None,
            fs: fat,
        }
    }

    pub fn update_size(&mut self)
    {
        ()
    }

    pub fn set_fst_cluster(&mut self, cluster: u32)
    {
        self.fst_cluster = Some(cluster);
        match self.entry 
        {
            Some(ref mut e) => e.set_fst_cluster(cluster),
            None => {}
        }
    }

    pub fn bytes_left_in_file(&self) -> Option<usize>
    {
        match self.entry
        {
            None => None,
            Some(ref ent) => Some(ent.size() as usize - self.offset)
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>
    {
        let cluster_size = self.fs.cluster_size;
        let cur_cluster = match self.cur_cluster {
            // End of chain is reached
            None => return Ok(0),
            Some(n) => {
                if self.offset % cluster_size == 0 
                {   
                    // Get next cluster
                    match self.fs.get_entry(n).unwrap()
                    {
                        Entry::Free | Entry::Bad => 
                            return Err(FileError::Error),
                        Entry::EndOfChain => return Ok(0),
                        Entry::Full(m) => m
                    }
                }
                else
                {
                    n
                }
            }
        };
        
        let offset_in_cluster = self.offset % cluster_size;
        let bytes_left_in_cluster = cluster_size - offset_in_cluster;
        let bytes_left_in_file = self.bytes_left_in_file()
            .unwrap_or(bytes_left_in_cluster);
        let read_size = min(
            min(buf.len(), 
                bytes_left_in_cluster),
            bytes_left_in_file);
        if read_size == 0 
        {
            return Ok(0)
        }
        
        let mut cluster_buf = vec![0; cluster_size];
        self.fs.read_cluster(&mut cluster_buf, cur_cluster);
        buf.clone_from_slice(
            &cluster_buf[offset_in_cluster .. offset_in_cluster + read_size]);
        Ok(read_size)
    }

    pub fn write(&mut self, buf: &mut [u8]) -> Result<usize, FileError>
    {
        let cluster_size = self.fs.cluster_size;
        let offset = self.offset % cluster_size;
        let bytes_left_in_cluster = cluster_size - offset;
        let write_size = min(buf.len(), bytes_left_in_cluster);

        if write_size == 0 {
            return Ok(0)
        }
        
        let cur_cluster = if offset == 0
        {
            // get next cluster
            let next_cluster = match self.cur_cluster 
            {
                None => self.fst_cluster,
                Some(n) => {
                    match self.fs.get_entry(n).unwrap()
                    {
                        Entry::Free | Entry::Bad => 
                            return Err(FileError::Error),
                        Entry::EndOfChain => None,
                        Entry::Full(m) => Some(m)
                    }
                }
            };

            match next_cluster
            {
                Some(n) => n,
                // A new cluster should be allocated
                None => 
                {
                    let new_cluster = self.fs.alloc_cluster(self.cur_cluster)
                                      .unwrap();
                    if self.fst_cluster.is_none()
                    {
                        self.set_fst_cluster(new_cluster);
                    }
                    // TODO: zero new directory cluster
                    new_cluster
                }
            }
        }
        else
        {
            match self.cur_cluster {
                Some(n) => n,
                None => panic!("Offset inside cluster but no cluster allocated"),
            }
        };

        let mut cluster_buf = vec![0; cluster_size];
        self.fs.read_cluster(&mut cluster_buf, cur_cluster);
        let write_slice = &mut cluster_buf[offset .. offset + write_size];
        write_slice.clone_from_slice(&buf[0 .. write_size]);
        
        self.offset += write_size;
        self.cur_cluster = Some(cur_cluster);
        self.update_size();
        Ok(write_size)
    }
}

