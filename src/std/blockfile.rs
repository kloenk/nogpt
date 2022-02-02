use crate::BLOCK_SIZE;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::Path;

#[cfg(target_family = "windows")]
use std::os::windows::fs::FileExt;

#[cfg(target_family = "unix")]
use std::os::unix::fs::FileExt;

use block_device::BlockDevice;
pub struct BlockFile {
    inner: std::fs::File,
}

impl BlockFile {
    pub fn open<P: AsRef<Path>>(path: &P) -> Result<Self, Error> {
        std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .map(|f| f.into())
    }
}

impl BlockDevice for BlockFile {
    type Error = Error;

    fn read(
        &self,
        buf: &mut [u8],
        address: usize,
        number_of_blocks: usize,
    ) -> Result<(), Self::Error> {
        #[cfg(target_family = "unix")]
        let read = self
            .inner
            .read_at(buf, BLOCK_SIZE as u64 * address as u64)?;

        #[cfg(target_family = "windows")]
        let read = self.inner.seek_read(buf, BLOCK_SIZE * address as u64)?;

        /*if read != number_of_blocks {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }*/

        Ok(())
    }

    fn write(
        &self,
        buf: &[u8],
        address: usize,
        number_of_blocks: usize,
    ) -> Result<(), Self::Error> {
        #[cfg(target_family = "unix")]
        let write = self
            .inner
            .write_at(buf, BLOCK_SIZE as u64 * address as u64)?;

        #[cfg(target_family = "windows")]
        let write = self.inner.seek_write(buf, BLOCK_SIZE * address as u64)?;

        /*if write != number_of_blocks {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }*/

        Ok(())
    }
}

impl From<File> for BlockFile {
    fn from(f: File) -> Self {
        Self { inner: f }
    }
}
