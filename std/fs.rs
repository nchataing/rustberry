use syscall;
//use syscall::OpenFlags;
use io;
use io::{Read, Write, Seek, SeekFrom};

pub struct File
{
    descr: syscall::FileDescriptor,
}

impl File
{
    pub fn open(path: &str/*, flags: OpenFlags*/) -> io::Result<File>
    {
        // TODO: Better errors
        let descr = syscall::open(path)
            .ok_or(io::Error { kind: io::ErrorKind::NotFound,
                               error: "could not open file" })?;

        Ok(File { descr })
    }
}

impl Read for File
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
    {
        Ok(syscall::read(self.descr, buf))
    }
}

impl Write for File
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>
    {
        Ok(syscall::write(self.descr, buf))
    }

    fn flush(&mut self) -> io::Result<()>
    {
        // No buffer inside File
        Ok(())
    }
}

impl Seek for File
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64>
    {
        Ok(syscall::seek(self.descr, pos))
    }
}

impl Drop for File
{
    fn drop(&mut self)
    {
        syscall::close(self.descr)
    }
}
