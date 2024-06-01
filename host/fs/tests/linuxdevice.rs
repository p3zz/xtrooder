//! Helpers for using embedded-sdmmc on Linux

use chrono::Timelike;
use fs::blockdevice::{BlockCount, BlockDevice, BlockIdx, BlockTrait};
use fs::filesystem::timestamp::{TimeSource, Timestamp};
use fs::{DeviceError, BLOCK_LEN};
use async_std::fs::{File, OpenOptions};
use async_std::io::prelude::*;
use async_std::io::SeekFrom;
use async_std::path::Path;
use std::cell::RefCell;

#[derive(Clone)]
pub struct Block {
    pub inner: [u8; BLOCK_LEN as usize],
}

impl BlockTrait for Block {
    fn new() -> Self {
        Self {
            inner: [0u8; BLOCK_LEN as usize],
        }
    }

    fn content_mut(&mut self) -> &mut [u8; 512] {
        &mut self.inner
    }

    fn content(&self) -> &[u8; 512] {
        &self.inner
    }
}

#[derive(Debug)]
pub struct LinuxBlockDevice {
    file: RefCell<File>,
}

impl LinuxBlockDevice {
    pub async fn new<P>(filename: P) -> Result<LinuxBlockDevice, std::io::Error>
    where
        P: AsRef<Path>,
    {
        Ok(LinuxBlockDevice {
            file: RefCell::new(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(filename).await?,
            )
        })
    }
}

impl BlockDevice for LinuxBlockDevice {
    type B = Block;
    type E = std::io::Error;

    async fn read(
        &mut self,
        blocks: &mut [Self::B],
        start_block_idx: BlockIdx,
    ) -> Result<(), DeviceError<Self::E>> {
        self.file
            .borrow_mut()
            .seek(SeekFrom::Start(start_block_idx.into_bytes())).await.map_err(|e|DeviceError::DeviceError(e))?;
        for block in blocks.iter_mut() {
            self.file.borrow_mut().read_exact(block.content_mut()).await.map_err(|e|DeviceError::DeviceError(e))?;
            println!(
                "Read block {:?}",
                start_block_idx
            );
        }
        Ok(())
    }

    async fn write(&mut self, blocks: &[Self::B], start_block_idx: BlockIdx) -> Result<(), DeviceError<Self::E>> {
        self.file
            .borrow_mut()
            .seek(SeekFrom::Start(start_block_idx.into_bytes())).await.map_err(|e|DeviceError::DeviceError(e))?;
        for block in blocks.iter() {
            self.file.borrow_mut().write_all(block.content()).await.map_err(|e|DeviceError::DeviceError(e))?;
            println!("Wrote: {:?}", start_block_idx);
        }
        Ok(())
    }

    async fn num_blocks(&self) -> Result<BlockCount, DeviceError<Self::E>> {
        let num_blocks = self.file.borrow().metadata().await.unwrap().len() / 512;
        Ok(BlockCount(num_blocks as u32))
    }
}

#[derive(Debug)]
pub struct Clock;

impl TimeSource for Clock {
    fn get_timestamp(&self) -> Timestamp {
        use chrono::Datelike;
        let local: chrono::DateTime<chrono::Local> = chrono::Local::now();
        Timestamp {
            year_since_1970: (local.year() - 1970) as u8,
            zero_indexed_month: local.month0() as u8,
            zero_indexed_day: local.day0() as u8,
            hours: local.hour() as u8,
            minutes: local.minute() as u8,
            seconds: local.second() as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn always_passes(){
        assert!(true);
    }
}
// ****************************************************************************
//
// End Of File
//
// ****************************************************************************