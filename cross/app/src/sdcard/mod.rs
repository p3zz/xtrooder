use embassy_stm32::sdmmc::{DataBlock, Error, Instance, Sdmmc, SdmmcDma};
use fs::blockdevice::{BlockCount, BlockDevice, BlockIdx, BlockTrait};
use fs::DeviceError;
use fs::BLOCK_LEN;

#[derive(Clone)]
pub struct Block {
    pub inner: DataBlock,
}

impl BlockTrait for Block {
    fn new() -> Self {
        Self {
            inner: DataBlock([0u8; BLOCK_LEN as usize]),
        }
    }

    fn content_mut(&mut self) -> &mut [u8; 512] {
        &mut self.inner.0
    }

    fn content(&self) -> &[u8; 512] {
        &self.inner.0
    }
}

pub struct SdmmcDevice<'d, T: Instance, Dma: SdmmcDma<T> + 'd> {
    inner: Sdmmc<'d, T, Dma>,
}

impl<'d, T: Instance, Dma: SdmmcDma<T> + 'd> SdmmcDevice<'d, T, Dma> {
    pub fn new(inner: Sdmmc<'d, T, Dma>) -> Self {
        Self { inner }
    }
}

impl<'d, T: Instance, Dma: SdmmcDma<T> + 'd> BlockDevice for SdmmcDevice<'d, T, Dma> {
    type B = Block;
    type E = Error;
    /// Read one or more blocks, starting at the given block index.
    async fn read(
        &mut self,
        blocks: &mut [Self::B],
        start_block_idx: BlockIdx,
    ) -> Result<(), DeviceError<Self::E>> {
        for block in blocks.iter_mut() {
            self.inner
                .read_block(start_block_idx.0, &mut block.inner)
                .await
                .map_err(DeviceError::DeviceError)?;
        }
        Ok(())
    }
    /// Write one or more blocks, starting at the given block index.
    async fn write(
        &mut self,
        blocks: &[Self::B],
        start_block_idx: BlockIdx,
    ) -> Result<(), DeviceError<Self::E>> {
        for block in blocks.iter() {
            self.inner
                .write_block(start_block_idx.0, &block.inner)
                .await
                .map_err(DeviceError::DeviceError)?;
        }
        Ok(())
    }
    /// Determine how many blocks this device can hold.
    fn num_blocks(&self) -> Result<BlockCount, DeviceError<Self::E>> {
        let count = self
            .inner
            .card()
            .map_err(DeviceError::DeviceError)?
            .csd
            .block_count();
        Ok(BlockCount(count))
    }
}

/// Represents a block device - a device which can read and write blocks (or
/// sectors). Only supports devices which are <= 2 TiB in size.
impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

impl core::ops::Deref for Block {
    type Target = [u8; 512];
    fn deref(&self) -> &[u8; 512] {
        self.inner.deref()
    }
}

impl core::ops::DerefMut for Block {
    fn deref_mut(&mut self) -> &mut [u8; 512] {
        self.inner.deref_mut()
    }
}

impl core::fmt::Debug for Block {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(fmt, "Block:")?;
        for line in self.inner.chunks(32) {
            for b in line {
                write!(fmt, "{:02x}", b)?;
            }
            write!(fmt, " ")?;
            for &b in line {
                if (0x20..=0x7F).contains(&b) {
                    write!(fmt, "{}", b as char)?;
                } else {
                    write!(fmt, ".")?;
                }
            }
            writeln!(fmt)?;
        }
        Ok(())
    }
}
