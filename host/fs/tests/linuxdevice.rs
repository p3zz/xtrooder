//! Helpers for using embedded-sdmmc on Linux

use chrono::Timelike;
use fs::blockdevice::{BlockCount, BlockDevice, BlockIdx, BlockTrait};
use fs::filesystem::timestamp::{TimeSource, Timestamp};
use fs::{DeviceError, BLOCK_LEN};
use async_std::fs::{File, OpenOptions};
use async_std::io::prelude::*;
use async_std::io::SeekFrom;
use async_std::path::Path;
use hex_literal::hex;
use std::cell::RefCell;

const MBR: [Block;3] = [
    Block {
        inner: hex!("
            fa b8 00 10 8e d0 bc 00 b0 b8 00 00 8e d8 8e c0
            fb be 00 7c bf 00 06 b9 00 02 f3 a4 ea 21 06 00
            00 be be 07 38 04 75 0b 83 c6 10 81 fe fe 07 75
            f3 eb 16 b4 02 b0 01 bb 00 7c b2 80 8a 74 01 8b
            4c 02 cd 13 ea 00 7c 00 00 eb fe 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 4c ca de 06 00 00 00 04
            01 04 0c fe c2 ff 01 00 00 00 33 22 11 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 aa
        "),
    },
    Block {
        inner: hex!("
            eb 58 90 6d 6b 66 73 2e 66 61 74 00 02 08 20 00
            02 00 00 00 00 f8 00 00 10 00 04 00 00 08 00 00
            00 20 76 00 80 1d 00 00 00 00 00 00 02 00 00 00
            01 00 06 00 00 00 00 00 00 00 00 00 00 00 00 00
            80 01 29 0b a8 89 27 50 69 63 74 75 72 65 73 20
            20 20 46 41 54 33 32 20 20 20 0e 1f be 77 7c ac
            22 c0 74 0b 56 b4 0e bb 07 00 cd 10 5e eb f0 32
            e4 cd 16 cd 19 eb fe 54 68 69 73 20 69 73 20 6e
            6f 74 20 61 20 62 6f 6f 74 61 62 6c 65 20 64 69
            73 6b 2e 20 20 50 6c 65 61 73 65 20 69 6e 73 65
            72 74 20 61 20 62 6f 6f 74 61 62 6c 65 20 66 6c
            6f 70 70 79 20 61 6e 64 0d 0a 70 72 65 73 73 20
            61 6e 79 20 6b 65 79 20 74 6f 20 74 72 79 20 61
            67 61 69 6e 20 2e 2e 2e 20 0d 0a 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 aa
        "),
    },
    Block {
        inner: hex!("
            52 52 61 41 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
            00 00 00 00 72 72 41 61 FF FF FF FF FF FF FF FF
            00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 AA
        "),
    }
];

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

    async fn format_f32(&mut self){
        for block in MBR{
            self.file.borrow_mut().write_all(block.content()).await.unwrap()
        }
    }

    pub async fn clear(&mut self){
        self.file.borrow_mut().set_len(0).await.unwrap()
    }

    pub async fn initialize(&mut self, num_blocks: BlockCount){
        self.clear().await;
        let mut b = Block{inner: [0u8; BLOCK_LEN as usize]};
        b.content_mut()[BLOCK_LEN as usize - 2] = 0x55;
        b.content_mut()[BLOCK_LEN as usize - 1] = 0xAA;
        
        for _ in 0..num_blocks.0{
            self.file.borrow_mut().write(b.content()).await.unwrap();
        }
        self.file.borrow_mut().seek(SeekFrom::Start(0)).await.unwrap();
        self.format_f32().await;
        self.file.borrow_mut().seek(SeekFrom::Start(0)).await.unwrap();
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
        self.file.borrow_mut().flush().await.unwrap();
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
    use async_std::{io::ReadExt, path::Path};
    use fs::{blockdevice::{BlockDevice, BlockIdx, BlockTrait}, filesystem::files::Mode, volume_mgr::{VolumeIdx, VolumeManager}, BLOCK_LEN};

    use crate::{Block, Clock, LinuxBlockDevice};

    #[tokio::test]
    async fn file_create_close_open() {
        let file_path = Path::new("assets/linuxdevice.txt");
        let mut device = LinuxBlockDevice::new(file_path).await.unwrap();
        device.initialize(fs::blockdevice::BlockCount(20000)).await;
        let mut c = VolumeManager::new(device, Clock);
        let mut v = c.open_volume(VolumeIdx(0)).await.unwrap();
        let mut root_dir = v.open_root_dir().unwrap();
        let filename = "test.txt";
        let f = root_dir.open_file_in_dir(filename, Mode::ReadWriteCreate).await.unwrap();
        f.close().await.unwrap();
        let f = root_dir.open_file_in_dir(filename, Mode::ReadOnly).await;
        assert!(f.is_ok());
    }

    #[tokio::test]
    async fn file_create_close_open_2() {
        let file_path = Path::new("assets/linuxdevice.txt");
        let mut device = LinuxBlockDevice::new(file_path).await.unwrap();
        device.initialize(fs::blockdevice::BlockCount(20000)).await;
        let mut c = VolumeManager::new(device, Clock);
        let v = c.open_raw_volume(VolumeIdx(0)).await.unwrap();
        let root_dir = c.open_root_dir(v).unwrap();
        let filename = "test.txt";
        let f = c.open_file_in_dir(root_dir, filename, Mode::ReadWriteCreate).await.unwrap();
        c.close_file(f).await.unwrap();
        let f = c.open_file_in_dir(root_dir, filename, Mode::ReadOnly).await;
        assert!(f.is_ok());
    }

    #[tokio::test]
    async fn file_create_write_close_open_read() {
        let file_path = Path::new("assets/linuxdevice.txt");
        let mut device = LinuxBlockDevice::new(file_path).await.unwrap();
        device.initialize(fs::blockdevice::BlockCount(20000)).await;
        let mut c = VolumeManager::new(device, Clock);
        let mut v = c.open_volume(VolumeIdx(0)).await.unwrap();
        let mut root_dir = v.open_root_dir().unwrap();
        let filename = "test.txt";
        let mut f = root_dir.open_file_in_dir(filename, Mode::ReadWriteCreate).await.unwrap();
        let buffer_to_write = [0x1,0x2, 0x3];
        f.write(&buffer_to_write).await.unwrap();
        f.close().await.unwrap();
        let mut f = root_dir.open_file_in_dir(filename, Mode::ReadOnly).await.unwrap();
        let mut buffer_to_read = [0u8; 3];
        let num_read = f.read(&mut buffer_to_read).await.unwrap();
        assert_eq!(num_read, 3);
        assert_eq!(buffer_to_read.get(0).unwrap(), buffer_to_write.get(0).unwrap());
        assert_eq!(buffer_to_read.get(1).unwrap(), buffer_to_write.get(1).unwrap());
        assert_eq!(buffer_to_read.get(2).unwrap(), buffer_to_write.get(2).unwrap());
    }

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