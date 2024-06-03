use crate::{DeviceError, BLOCK_LEN};

// Block Device support
//
// Generic code for handling block devices, such as types for identifying
// a particular block on a block device by its index.

/// Represents a standard 512 byte block (also known as a sector). IBM PC
/// formatted 5.25" and 3.5" floppy disks, SD/MMC cards up to 1 GiB in size
/// and IDE/SATA Hard Drives up to about 2 TiB all have 512 byte blocks.
///
/// This library does not support devices with a block size other than 512
/// bytes.

pub trait BlockTrait {
    fn copy_from_slice(slice: &[u8; BLOCK_LEN as usize]) -> Self;

    fn new() -> Self;

    fn content_mut(&mut self) -> &mut [u8; BLOCK_LEN as usize];

    fn content(&self) -> &[u8; BLOCK_LEN as usize];
}

/// Represents the linear numeric address of a block (or sector). The first
/// block on a disk gets `BlockIdx(0)` (which usually contains the Master Boot
/// Record).
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockIdx(pub u32);

/// Represents the a number of blocks (or sectors). Add this to a `BlockIdx`
/// to get an actual address on disk.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockCount(pub u32);

/// An iterator returned from `Block::range`.
pub struct BlockIter {
    inclusive_end: BlockIdx,
    current: BlockIdx,
}

/// Represents a block device - a device which can read and write blocks (or
/// sectors). Only supports devices which are <= 2 TiB in size.
pub trait BlockDevice {
    type B: BlockTrait;
    type E: core::fmt::Debug;
    /// Read one or more blocks, starting at the given block index.
    async fn read(
        &mut self,
        blocks: &mut [Self::B],
        start_block_idx: BlockIdx,
    ) -> Result<(), DeviceError<Self::E>>;
    /// Write one or more blocks, starting at the given block index.
    async fn write(
        &mut self,
        blocks: &[Self::B],
        start_block_idx: BlockIdx,
    ) -> Result<(), DeviceError<Self::E>>;
    /// Determine how many blocks this device can hold.
    async fn num_blocks(&self) -> Result<BlockCount, DeviceError<Self::E>>;
}

impl core::ops::Add<BlockCount> for BlockIdx {
    type Output = BlockIdx;
    fn add(self, rhs: BlockCount) -> BlockIdx {
        BlockIdx(self.0 + rhs.0)
    }
}

impl core::ops::AddAssign<BlockCount> for BlockIdx {
    fn add_assign(&mut self, rhs: BlockCount) {
        self.0 += rhs.0
    }
}

impl core::ops::Add<BlockCount> for BlockCount {
    type Output = BlockCount;
    fn add(self, rhs: BlockCount) -> BlockCount {
        BlockCount(self.0 + rhs.0)
    }
}

impl core::ops::AddAssign<BlockCount> for BlockCount {
    fn add_assign(&mut self, rhs: BlockCount) {
        self.0 += rhs.0
    }
}

impl core::ops::Sub<BlockCount> for BlockIdx {
    type Output = BlockIdx;
    fn sub(self, rhs: BlockCount) -> BlockIdx {
        BlockIdx(self.0 - rhs.0)
    }
}

impl core::ops::SubAssign<BlockCount> for BlockIdx {
    fn sub_assign(&mut self, rhs: BlockCount) {
        self.0 -= rhs.0
    }
}

impl core::ops::Sub<BlockCount> for BlockCount {
    type Output = BlockCount;
    fn sub(self, rhs: BlockCount) -> BlockCount {
        BlockCount(self.0 - rhs.0)
    }
}

impl core::ops::SubAssign<BlockCount> for BlockCount {
    fn sub_assign(&mut self, rhs: BlockCount) {
        self.0 -= rhs.0
    }
}

impl BlockIdx {
    /// Convert a block index into a 64-bit byte offset from the start of the
    /// volume. Useful if your underlying block device actually works in
    /// bytes, like `open("/dev/mmcblk0")` does on Linux.
    pub fn into_bytes(self) -> u64 {
        (u64::from(self.0)) * (BLOCK_LEN as u64)
    }

    /// Create an iterator from the current `BlockIdx` through the given
    /// number of blocks.
    pub fn range(self, num: BlockCount) -> BlockIter {
        BlockIter::new(self, self + BlockCount(num.0))
    }
}

impl BlockCount {
    pub const fn from_bytes(byte_count: u32) -> BlockCount {
        let mut count = byte_count / BLOCK_LEN;
        if (count * BLOCK_LEN) != byte_count {
            count += 1;
        }
        BlockCount(count)
    }

    /// Take a number of blocks and increment by the integer number of blocks
    /// required to get to the block that holds the byte at the given offset.
    pub fn offset_bytes(self, offset: u32) -> Self {
        BlockCount(self.0 + (offset / BLOCK_LEN))
    }
}

impl BlockIter {
    /// Create a new `BlockIter`, from the given start block, through (and
    /// including) the given end block.
    pub const fn new(start: BlockIdx, inclusive_end: BlockIdx) -> BlockIter {
        BlockIter {
            inclusive_end,
            current: start,
        }
    }
}

impl core::iter::Iterator for BlockIter {
    type Item = BlockIdx;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.0 >= self.inclusive_end.0 {
            None
        } else {
            let this = self.current;
            self.current += BlockCount(1);
            Some(this)
        }
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
