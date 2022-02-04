use std::fs::File;
use std::io::Error;
use std::path::Path;

#[cfg(target_family = "windows")]
use std::os::windows::fs::FileExt;

#[cfg(target_family = "unix")]
use std::os::unix::fs::FileExt;

use block_device::BlockDevice;
pub struct BlockFile<const N: u32> {
    inner: std::fs::File,
}

impl<const N: u32> BlockFile<N> {
    pub fn open<P: AsRef<Path>>(path: &P) -> Result<Self, Error> {
        std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .map(|f| f.into())
    }
}

impl<const N: u32> BlockDevice for BlockFile<N> {
    const BLOCK_SIZE: u32 = N;
    type Error = Error;

    fn read(
        &self,
        buf: &mut [u8],
        address: usize,
        _number_of_blocks: usize,
    ) -> Result<(), Self::Error> {
        #[cfg(target_family = "unix")]
        self.inner
            .read_at(buf, Self::BLOCK_SIZE as u64 * address as u64)?;

        #[cfg(target_family = "windows")]
        let read = self
            .inner
            .seek_read(buf, Self::BLOCK_SIZE * address as u64)?;

        /*if read != number_of_blocks {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }*/

        Ok(())
    }

    fn write(
        &self,
        buf: &[u8],
        address: usize,
        _number_of_blocks: usize,
    ) -> Result<(), Self::Error> {
        #[cfg(target_family = "unix")]
        self.inner
            .write_at(buf, Self::BLOCK_SIZE as u64 * address as u64)?;

        #[cfg(target_family = "windows")]
        let write = self
            .inner
            .seek_write(buf, Self::BLOCK_SIZE * address as u64)?;

        /*if write != number_of_blocks {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }*/

        Ok(())
    }
}

impl<const N: u32> From<File> for BlockFile<N> {
    fn from(f: File) -> Self {
        Self { inner: f }
    }
}
