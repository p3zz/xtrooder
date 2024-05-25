//! FAT16/FAT32 file system implementation
//!
//! Implements the File Allocation Table file system. Supports FAT16 and FAT32 volumes.

/// Number of entries reserved at the start of a File Allocation Table
pub const RESERVED_ENTRIES: u32 = 2;

/// Indentifies the supported types of FAT format
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FatType {
    /// FAT16 Format
    Fat16,
    /// FAT32 Format
    Fat32,
}

pub struct BlockCache {
    block: Block,
    idx: Option<BlockIdx>,
}
impl BlockCache {
    pub fn empty() -> Self {
        BlockCache {
            block: Block::new(),
            idx: None,
        }
    }
    pub(crate) async fn read<D: BlockDevice>(
        &mut self,
        block_device: &mut D,
        block_idx: BlockIdx,
    ) -> Result<&Block, DeviceError>
    where
        D: BlockDevice,
    {
        if Some(block_idx) != self.idx {
            self.idx = Some(block_idx);
            block_device
                .read(core::slice::from_mut(&mut self.block), block_idx)
                .await?;
        }
        Ok(&self.block)
    }
}

pub mod bpb;
pub mod info;
pub mod ondiskdirentry;
pub mod volume;

use crate::{
    blockdevice::{Block, BlockDevice, BlockIdx},
    DeviceError,
};

// ****************************************************************************
//
// Unit Tests
//
// ****************************************************************************

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
