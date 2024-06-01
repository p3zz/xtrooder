//! The Volume Manager implementation.
//!
//! The volume manager handles partitions and open files on a block device.

use crate::{
    blockdevice::{BlockDevice, BlockTrait},
    DeviceError, BLOCK_LEN,
};
use byteorder::{ByteOrder, LittleEndian};

use crate::{
    blockdevice::{BlockCount, BlockIdx},
    fat::{
        ondiskdirentry::OnDiskDirEntry,
        volume::{parse_volume, FatVolume},
        BlockCache, FatType, RESERVED_ENTRIES,
    },
    filesystem::{
        attributes::Attributes,
        cluster::ClusterId,
        directory::{DirEntry, Directory, DirectoryInfo, RawDirectory},
        filename::{ShortFileName, ToShortFileName},
        files::{FileInfo, Mode, RawFile},
        search_id::{SearchId, SearchIdGenerator},
        timestamp::TimeSource,
        MAX_FILE_SIZE,
    },
    PARTITION_ID_FAT16, PARTITION_ID_FAT16_LBA, PARTITION_ID_FAT32_CHS_LBA, PARTITION_ID_FAT32_LBA,
};

use heapless::Vec;

const PARTITION1_START: usize = 446;
const PARTITION2_START: usize = PARTITION1_START + PARTITION_INFO_LENGTH;
const PARTITION3_START: usize = PARTITION2_START + PARTITION_INFO_LENGTH;
const PARTITION4_START: usize = PARTITION3_START + PARTITION_INFO_LENGTH;
const FOOTER_START: usize = 510;
const FOOTER_VALUE: u16 = 0xAA55;
const PARTITION_INFO_LENGTH: usize = 16;
const PARTITION_INFO_STATUS_INDEX: usize = 0;
const PARTITION_INFO_TYPE_INDEX: usize = 4;
const PARTITION_INFO_LBA_START_INDEX: usize = 8;
const PARTITION_INFO_NUM_BLOCKS_INDEX: usize = 12;

/// Represents a partition with a filesystem within it.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RawVolume(SearchId);

impl RawVolume {
    /// Convert a raw volume into a droppable [`Volume`]
    pub fn to_volume<
        D,
        T,
        const MAX_DIRS: usize,
        const MAX_FILES: usize,
        const MAX_VOLUMES: usize,
    >(
        self,
        volume_mgr: &mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    ) -> Volume<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
    where
        D: BlockDevice,
        T: TimeSource,
    {
        Volume::new(self, volume_mgr)
    }
}

/// Represents an open volume on disk.
///
/// In contrast to a `RawVolume`, a `Volume` holds a mutable reference to its
/// parent `VolumeManager`, which restricts which operations you can perform.
///
/// If you drop a value of this type, it closes the volume automatically, but
/// any error that may occur will be ignored. To handle potential errors, use
/// the [`Volume::close`] method.
pub struct Volume<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
where
    D: BlockDevice,
    T: TimeSource,
{
    raw_volume: RawVolume,
    volume_mgr: &'a mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
}

impl<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
    Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: BlockDevice,
    T: TimeSource,
{
    /// Create a new `Volume` from a `RawVolume`
    pub fn new(
        raw_volume: RawVolume,
        volume_mgr: &'a mut VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    ) -> Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES> {
        Volume {
            raw_volume,
            volume_mgr,
        }
    }

    /// Open the volume's root directory.
    ///
    /// You can then read the directory entries with `iterate_dir`, or you can
    /// use `open_file_in_dir`.
    pub fn open_root_dir(
        &mut self,
    ) -> Result<Directory<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>, DeviceError<D::E>> {
        let d = self.volume_mgr.open_root_dir(self.raw_volume)?;
        Ok(d.to_directory(self.volume_mgr))
    }

    /// Convert back to a raw volume
    pub fn to_raw_volume(self) -> RawVolume {
        let v = self.raw_volume;
        core::mem::forget(self);
        v
    }

    /// Consume the `Volume` handle and close it. The behavior of this is similar
    /// to using [`core::mem::drop`] or letting the `Volume` go out of scope,
    /// except this lets the user handle any errors that may occur in the process,
    /// whereas when using drop, any errors will be discarded silently.
    pub fn close(self) -> Result<(), DeviceError<D::E>> {
        let result = self.volume_mgr.close_volume(self.raw_volume);
        core::mem::forget(self);
        result
    }
}

impl<'a, D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize> Drop
    for Volume<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: BlockDevice,
    T: TimeSource,
{
    fn drop(&mut self) {
        _ = self.volume_mgr.close_volume(self.raw_volume)
    }
}

/// Internal information about a Volume
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct VolumeInfo {
    /// Search ID for this volume.
    volume_id: RawVolume,
    /// TODO: some kind of index
    idx: VolumeIdx,
    /// What kind of volume this is
    volume_type: VolumeType,
}

/// This enum holds the data for the various different types of filesystems we
/// support.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum VolumeType {
    /// FAT16/FAT32 formatted volumes.
    Fat(FatVolume),
}

/// A `VolumeIdx` is a number which identifies a volume (or partition) on a
/// disk.
///
/// `VolumeIdx(0)` is the first primary partition on an MBR partitioned disk.
#[cfg_attr(feature = "defmt-log", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct VolumeIdx(pub usize);

/// A `VolumeManager` wraps a block device and gives access to the FAT-formatted
/// volumes within it.
pub struct VolumeManager<
    D,
    T,
    const MAX_DIRS: usize = 4,
    const MAX_FILES: usize = 4,
    const MAX_VOLUMES: usize = 1,
> where
    D: BlockDevice,
    T: TimeSource,
{
    pub(crate) block_device: D,
    pub(crate) time_source: T,
    id_generator: SearchIdGenerator,
    open_volumes: Vec<VolumeInfo, MAX_VOLUMES>,
    open_dirs: Vec<DirectoryInfo, MAX_DIRS>,
    open_files: Vec<FileInfo, MAX_FILES>,
}

impl<D, T> VolumeManager<D, T, 4, 4>
where
    D: BlockDevice,
    T: TimeSource,
{
    /// Create a new Volume Manager using a generic `BlockDevice`. From this
    /// object we can open volumes (partitions) and with those we can open
    /// files.
    ///
    /// This creates a `VolumeManager` with default values
    /// MAX_DIRS = 4, MAX_FILES = 4, MAX_VOLUMES = 1. Call `VolumeManager::new_with_limits(block_device, time_source)`
    /// if you need different limits.
    pub fn new(block_device: D, time_source: T) -> VolumeManager<D, T, 4, 4, 1> {
        // Pick a random starting point for the IDs that's not zero, because
        // zero doesn't stand out in the logs.
        Self::new_with_limits(block_device, time_source, 5000)
    }
}

impl<D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>
    VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: BlockDevice,
    T: TimeSource,
{
    /// Create a new Volume Manager using a generic `BlockDevice`. From this
    /// object we can open volumes (partitions) and with those we can open
    /// files.
    ///
    /// You can also give an offset for all the IDs this volume manager
    /// generates, which might help you find the IDs in your logs when
    /// debugging.
    pub fn new_with_limits(
        block_device: D,
        time_source: T,
        id_offset: u32,
    ) -> VolumeManager<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES> {
        VolumeManager {
            block_device,
            time_source,
            id_generator: SearchIdGenerator::new(id_offset),
            open_volumes: Vec::new(),
            open_dirs: Vec::new(),
            open_files: Vec::new(),
        }
    }

    /// Temporarily get access to the underlying block device.
    pub fn device(&mut self) -> &mut D {
        &mut self.block_device
    }

    /// Get a volume (or partition) based on entries in the Master Boot Record.
    ///
    /// We do not support GUID Partition Table disks. Nor do we support any
    /// concept of drive letters - that is for a higher layer to handle.
    pub async fn open_volume(
        &mut self,
        volume_idx: VolumeIdx,
    ) -> Result<Volume<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>, DeviceError<D::E>> {
        let v = self.open_raw_volume(volume_idx).await?;
        Ok(v.to_volume(self))
    }

    /// Get a volume (or partition) based on entries in the Master Boot Record.
    ///
    /// We do not support GUID Partition Table disks. Nor do we support any
    /// concept of drive letters - that is for a higher layer to handle.
    ///
    /// This function gives you a `RawVolume` and you must close the volume by
    /// calling `VolumeManager::close_volume`.
    pub async fn open_raw_volume(
        &mut self,
        volume_idx: VolumeIdx,
    ) -> Result<RawVolume, DeviceError<D::E>> {

        if self.open_volumes.is_full() {
            // TODO replace error
            return Err(DeviceError::TooManyOpenVolumes);
        }

        for v in self.open_volumes.iter() {
            if v.idx == volume_idx {
                // TODO replace error
                return Err(DeviceError::VolumeAlreadyOpen);
            }
        }

        let (part_type, lba_start, num_blocks) = {
            let mut blocks = [D::B::new()];
            self.block_device.read(&mut blocks, BlockIdx(0)).await?;
            let block = &blocks[0];
            // We only support Master Boot Record (MBR) partitioned cards, not
            // GUID Partition Table (GPT)
            let content = block.content();
            if LittleEndian::read_u16(&content[FOOTER_START..FOOTER_START + 2]) != FOOTER_VALUE {
                // TODO replace error
                return Err(DeviceError::FormatError("Invalid MBR signature"));
            }
            let partition = match volume_idx {
                VolumeIdx(0) => {
                    &content[PARTITION1_START..(PARTITION1_START + PARTITION_INFO_LENGTH)]
                }
                VolumeIdx(1) => {
                    &content[PARTITION2_START..(PARTITION2_START + PARTITION_INFO_LENGTH)]
                }
                VolumeIdx(2) => {
                    &content[PARTITION3_START..(PARTITION3_START + PARTITION_INFO_LENGTH)]
                }
                VolumeIdx(3) => {
                    &content[PARTITION4_START..(PARTITION4_START + PARTITION_INFO_LENGTH)]
                }
                _ => {
                    // TODO replace error
                    return Err(DeviceError::NoSuchVolume);
                }
            };
            // Only 0x80 and 0x00 are valid (bootable, and non-bootable)
            if (partition[PARTITION_INFO_STATUS_INDEX] & 0x7F) != 0x00 {
                // TODO replace error
                return Err(DeviceError::FormatError("Invalid partition status"));
            }
            let lba_start = LittleEndian::read_u32(
                &partition[PARTITION_INFO_LBA_START_INDEX..(PARTITION_INFO_LBA_START_INDEX + 4)],
            );
            let num_blocks = LittleEndian::read_u32(
                &partition[PARTITION_INFO_NUM_BLOCKS_INDEX..(PARTITION_INFO_NUM_BLOCKS_INDEX + 4)],
            );
            (
                partition[PARTITION_INFO_TYPE_INDEX],
                BlockIdx(lba_start),
                BlockCount(num_blocks),
            )
        };
        match part_type {
            PARTITION_ID_FAT32_CHS_LBA
            | PARTITION_ID_FAT32_LBA
            | PARTITION_ID_FAT16_LBA
            | PARTITION_ID_FAT16 => {
                let volume = parse_volume(&mut self.block_device, lba_start, num_blocks).await?;
                let id = RawVolume(self.id_generator.get());
                let info = VolumeInfo {
                    volume_id: id,
                    idx: volume_idx,
                    volume_type: volume,
                };
                // We already checked for space
                self.open_volumes.push(info).unwrap();
                Ok(id)
            }
            // TODO replace error
            _ => Err(DeviceError::FormatError("Partition type not supported")),
        }
    }

    /// Open the volume's root directory.
    ///
    /// You can then read the directory entries with `iterate_dir`, or you can
    /// use `open_file_in_dir`.
    pub fn open_root_dir(&mut self, volume: RawVolume) -> Result<RawDirectory, DeviceError<D::E>> {
        // Opening a root directory twice is OK

        let directory_id = RawDirectory(self.id_generator.get());
        let dir_info = DirectoryInfo {
            volume_id: volume,
            cluster: ClusterId::ROOT_DIR,
            directory_id,
        };

        self.open_dirs
            .push(dir_info)
            .map_err(|_| DeviceError::TooManyOpenDirs)?;

        Ok(directory_id)
    }

    /// Open a directory.
    ///
    /// You can then read the directory entries with `iterate_dir` and `open_file_in_dir`.
    ///
    /// Passing "." as the name results in opening the `parent_dir` a second time.
    pub async fn open_dir<N>(
        &mut self,
        parent_dir: RawDirectory,
        name: N,
    ) -> Result<RawDirectory, DeviceError<D::E>>
    where
        N: ToShortFileName,
    {
        if self.open_dirs.is_full() {
            return Err(DeviceError::TooManyOpenDirs);
        }

        // Find dir by ID
        let parent_dir_idx = self.get_dir_by_id(parent_dir)?;
        let volume_idx = self.get_volume_by_id(self.open_dirs[parent_dir_idx].volume_id)?;
        let short_file_name = name
            .to_short_filename()
            .map_err(DeviceError::FilenameError)?;
        let parent_dir_info = &self.open_dirs[parent_dir_idx];

        // Open the directory
        if short_file_name == ShortFileName::this_dir() {
            // short-cut (root dir doesn't have ".")
            let directory_id = RawDirectory(self.id_generator.get());
            let dir_info = DirectoryInfo {
                directory_id,
                volume_id: self.open_volumes[volume_idx].volume_id,
                cluster: parent_dir_info.cluster,
            };

            self.open_dirs
                .push(dir_info)
                .map_err(|_| DeviceError::TooManyOpenDirs)?;

            return Ok(directory_id);
        }

        let dir_entry = match &self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => {
                fat.find_directory_entry(&mut self.block_device, parent_dir_info, &short_file_name)
                    .await?
            }
        };

        // debug!("Found dir entry: {:?}", dir_entry);

        if !dir_entry.attributes.is_directory() {
            return Err(DeviceError::OpenedFileAsDir);
        }

        // We don't check if the directory is already open - directories hold
        // no cached state and so opening a directory twice is allowable.

        // Remember this open directory.
        let directory_id = RawDirectory(self.id_generator.get());
        let dir_info = DirectoryInfo {
            directory_id,
            volume_id: self.open_volumes[volume_idx].volume_id,
            cluster: dir_entry.cluster,
        };

        self.open_dirs
            .push(dir_info)
            .map_err(|_| DeviceError::TooManyOpenDirs)?;

        Ok(directory_id)
    }

    /// Close a directory. You cannot perform operations on an open directory
    /// and so must close it if you want to do something with it.
    pub fn close_dir(&mut self, directory: RawDirectory) -> Result<(), DeviceError<D::E>> {
        for (idx, info) in self.open_dirs.iter().enumerate() {
            if directory == info.directory_id {
                self.open_dirs.swap_remove(idx);
                return Ok(());
            }
        }
        Err(DeviceError::BadHandle)
    }

    /// Close a volume
    ///
    /// You can't close it if there are any files or directories open on it.
    pub fn close_volume(&mut self, volume: RawVolume) -> Result<(), DeviceError<D::E>> {
        for f in self.open_files.iter() {
            if f.volume_id == volume {
                return Err(DeviceError::VolumeStillInUse);
            }
        }

        for d in self.open_dirs.iter() {
            if d.volume_id == volume {
                return Err(DeviceError::VolumeStillInUse);
            }
        }

        let volume_idx = self.get_volume_by_id(volume)?;
        self.open_volumes.swap_remove(volume_idx);

        Ok(())
    }

    /// Look in a directory for a named file.
    pub async fn find_directory_entry<N>(
        &mut self,
        directory: RawDirectory,
        name: N,
    ) -> Result<DirEntry, DeviceError<D::E>>
    where
        N: ToShortFileName,
    {
        let directory_idx = self.get_dir_by_id(directory)?;
        let volume_idx = self.get_volume_by_id(self.open_dirs[directory_idx].volume_id)?;
        match &self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => {
                let sfn = name
                    .to_short_filename()
                    .map_err(DeviceError::FilenameError)?;
                fat.find_directory_entry(
                    &mut self.block_device,
                    &self.open_dirs[directory_idx],
                    &sfn,
                )
                .await
            }
        }
    }

    /// Call a callback function for each directory entry in a directory.
    pub async fn iterate_dir<F>(
        &mut self,
        directory: RawDirectory,
        func: F,
    ) -> Result<(), DeviceError<D::E>>
    where
        F: FnMut(&DirEntry),
    {
        let directory_idx = self.get_dir_by_id(directory)?;
        let volume_idx = self.get_volume_by_id(self.open_dirs[directory_idx].volume_id)?;
        match &self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => {
                fat.iterate_dir(&mut self.block_device, &self.open_dirs[directory_idx], func)
                    .await
            }
        }
    }

    /// Open a file from a DirEntry. This is obtained by calling iterate_dir.
    ///
    /// # Safety
    ///
    /// The DirEntry must be a valid DirEntry read from disk, and not just
    /// random numbers.
    async unsafe fn open_dir_entry(
        &mut self,
        volume: RawVolume,
        dir_entry: DirEntry,
        mode: Mode,
    ) -> Result<RawFile, DeviceError<D::E>> {
        // This check is load-bearing - we do an unchecked push later.
        if self.open_files.is_full() {
            return Err(DeviceError::TooManyOpenFiles);
        }

        if dir_entry.attributes.is_read_only() && mode != Mode::ReadOnly {
            return Err(DeviceError::ReadOnly);
        }

        if dir_entry.attributes.is_directory() {
            return Err(DeviceError::OpenedDirAsFile);
        }

        // Check it's not already open
        if self.file_is_open(volume, &dir_entry) {
            return Err(DeviceError::FileAlreadyOpen);
        }

        let mode = solve_mode_variant(mode, true);
        let file_id = RawFile(self.id_generator.get());

        let file = match mode {
            Mode::ReadOnly => FileInfo {
                file_id,
                volume_id: volume,
                current_cluster: (0, dir_entry.cluster),
                current_offset: 0,
                mode,
                entry: dir_entry,
                dirty: false,
            },
            Mode::ReadWriteAppend => {
                let mut file = FileInfo {
                    file_id,
                    volume_id: volume,
                    current_cluster: (0, dir_entry.cluster),
                    current_offset: 0,
                    mode,
                    entry: dir_entry,
                    dirty: false,
                };
                // seek_from_end with 0 can't fail
                file.seek_from_end(0).ok();
                file
            }
            Mode::ReadWriteTruncate => {
                let mut file = FileInfo {
                    file_id,
                    volume_id: volume,
                    current_cluster: (0, dir_entry.cluster),
                    current_offset: 0,
                    mode,
                    entry: dir_entry,
                    dirty: false,
                };
                let volume_idx = self.get_volume_by_id(volume)?;
                match &mut self.open_volumes[volume_idx].volume_type {
                    VolumeType::Fat(fat) => {
                        fat.truncate_cluster_chain(&mut self.block_device, file.entry.cluster)
                            .await?
                    }
                };
                file.update_length(0);
                match &self.open_volumes[volume_idx].volume_type {
                    VolumeType::Fat(fat) => {
                        file.entry.mtime = self.time_source.get_timestamp();
                        fat.write_entry_to_disk(&mut self.block_device, &file.entry)
                            .await?;
                    }
                };

                file
            }
            _ => return Err(DeviceError::Unsupported),
        };

        // Remember this open file - can't be full as we checked already
        unsafe {
            self.open_files.push_unchecked(file);
        }

        Ok(file_id)
    }

    /// Open a file with the given full path. A file can only be opened once.
    pub async fn open_file_in_dir<N>(
        &mut self,
        directory: RawDirectory,
        name: N,
        mode: Mode,
    ) -> Result<RawFile, DeviceError<D::E>>
    where
        N: ToShortFileName,
    {
        // This check is load-bearing - we do an unchecked push later.
        if self.open_files.is_full() {
            return Err(DeviceError::TooManyOpenFiles);
        }

        let directory_idx = self.get_dir_by_id(directory)?;
        let directory_info = &self.open_dirs[directory_idx];
        let volume_id = self.open_dirs[directory_idx].volume_id;
        let volume_idx = self.get_volume_by_id(volume_id)?;
        #[cfg(test)]
        print!("Ciao");
        let volume_info = &self.open_volumes[volume_idx];
        let sfn = name
            .to_short_filename()
            .map_err(DeviceError::FilenameError)?;

        let dir_entry = match &volume_info.volume_type {
            VolumeType::Fat(fat) => {
                fat.find_directory_entry(&mut self.block_device, directory_info, &sfn)
                    .await
            }
        };

        let dir_entry = match dir_entry {
            Ok(entry) => {
                // we are opening an existing file
                Some(entry)
            }
            Err(_)
                if (mode == Mode::ReadWriteCreate)
                    | (mode == Mode::ReadWriteCreateOrTruncate)
                    | (mode == Mode::ReadWriteCreateOrAppend) =>
            {
                // We are opening a non-existant file, but that's OK because they
                // asked us to create it
                None
            }
            _ => {
                // We are opening a non-existant file, and that's not OK.
                return Err(DeviceError::NotFound);
            }
        };

        // Check if it's open already
        if let Some(dir_entry) = &dir_entry {
            if self.file_is_open(volume_info.volume_id, dir_entry) {
                return Err(DeviceError::FileAlreadyOpen);
            }
        }

        let mode = solve_mode_variant(mode, dir_entry.is_some());

        match mode {
            Mode::ReadWriteCreate => {
                if dir_entry.is_some() {
                    return Err(DeviceError::FileAlreadyExists);
                }
                let att = Attributes::create_from_fat(0);
                let volume_idx = self.get_volume_by_id(volume_id)?;
                let entry = match &mut self.open_volumes[volume_idx].volume_type {
                    VolumeType::Fat(fat) => {
                        fat.write_new_directory_entry(
                            &mut self.block_device,
                            &self.time_source,
                            directory_info.cluster,
                            sfn,
                            att,
                        )
                        .await?
                    }
                };

                let file_id = RawFile(self.id_generator.get());

                let file = FileInfo {
                    file_id,
                    volume_id,
                    current_cluster: (0, entry.cluster),
                    current_offset: 0,
                    mode,
                    entry,
                    dirty: false,
                };

                // Remember this open file - can't be full as we checked already
                unsafe {
                    self.open_files.push_unchecked(file);
                }

                Ok(file_id)
            }
            _ => {
                // Safe to unwrap, since we actually have an entry if we got here
                let dir_entry = dir_entry.unwrap();
                // Safety: We read this dir entry off disk and didn't change it
                unsafe { self.open_dir_entry(volume_id, dir_entry, mode).await }
            }
        }
    }

    /// Delete a closed file with the given filename, if it exists.
    pub async fn delete_file_in_dir<N>(
        &mut self,
        directory: RawDirectory,
        name: N,
    ) -> Result<(), DeviceError<D::E>>
    where
        N: ToShortFileName,
    {
        let dir_idx = self.get_dir_by_id(directory)?;
        let dir_info = &self.open_dirs[dir_idx];
        let volume_idx = self.get_volume_by_id(dir_info.volume_id)?;
        let sfn = name
            .to_short_filename()
            .map_err(DeviceError::FilenameError)?;

        let dir_entry = match &self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => {
                fat.find_directory_entry(&mut self.block_device, dir_info, &sfn)
                    .await
            }
        }?;

        if dir_entry.attributes.is_directory() {
            return Err(DeviceError::DeleteDirAsFile);
        }

        if self.file_is_open(dir_info.volume_id, &dir_entry) {
            return Err(DeviceError::FileAlreadyOpen);
        }

        let volume_idx = self.get_volume_by_id(dir_info.volume_id)?;
        match &self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => {
                fat.delete_directory_entry(&mut self.block_device, dir_info, &sfn)
                    .await?
            }
        }

        Ok(())
    }

    /// Check if a file is open
    ///
    /// Returns `true` if it's open, `false`, otherwise.
    fn file_is_open(&self, volume: RawVolume, dir_entry: &DirEntry) -> bool {
        for f in self.open_files.iter() {
            if f.volume_id == volume
                && f.entry.entry_block == dir_entry.entry_block
                && f.entry.entry_offset == dir_entry.entry_offset
            {
                return true;
            }
        }
        false
    }

    pub async fn read_byte(&mut self, file: RawFile) -> Result<u8, DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        let volume_idx = self.get_volume_by_id(self.open_files[file_idx].volume_id)?;
        // Calculate which file block the current offset lies within
        // While there is more to read, read the block and copy in to the buffer.
        // If we need to find the next cluster, walk the FAT.
        if self.open_files[file_idx].eof() {
            return Err(DeviceError::EndOfFile);
        }

        let mut current_cluster = self.open_files[file_idx].current_cluster;
        let (block_idx, block_offset, block_avail) = self
            .find_data_on_disk(
                volume_idx,
                &mut current_cluster,
                self.open_files[file_idx].current_offset,
            )
            .await?;
        self.open_files[file_idx].current_cluster = current_cluster;
        let mut blocks = [D::B::new()];
        self.block_device.read(&mut blocks, block_idx).await?;

        if block_avail == 0 || self.open_files[file_idx].left() == 0 {
            return Err(DeviceError::EndOfFile);
        }

        let b = blocks[0].content()[block_offset + 1];
        self.open_files[file_idx]
            .seek_from_current(1 as i32)
            .unwrap();

        Ok(b)
    }

    /// Read from an open file.
    pub async fn read(
        &mut self,
        file: RawFile,
        buffer: &mut [u8],
    ) -> Result<usize, DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        let volume_idx = self.get_volume_by_id(self.open_files[file_idx].volume_id)?;
        // Calculate which file block the current offset lies within
        // While there is more to read, read the block and copy in to the buffer.
        // If we need to find the next cluster, walk the FAT.
        let mut space = buffer.len();
        let mut read = 0;
        while space > 0 && !self.open_files[file_idx].eof() {
            let mut current_cluster = self.open_files[file_idx].current_cluster;
            let (block_idx, block_offset, block_avail) = self
                .find_data_on_disk(
                    volume_idx,
                    &mut current_cluster,
                    self.open_files[file_idx].current_offset,
                )
                .await?;
            self.open_files[file_idx].current_cluster = current_cluster;
            let mut blocks = [D::B::new()];
            self.block_device.read(&mut blocks, block_idx).await?;
            let block = &blocks[0];
            let to_copy = block_avail
                .min(space)
                .min(self.open_files[file_idx].left() as usize);
            assert!(to_copy != 0);
            buffer[read..read + to_copy]
                .copy_from_slice(&block.content()[block_offset..block_offset + to_copy]);
            read += to_copy;
            space -= to_copy;
            self.open_files[file_idx]
                .seek_from_current(to_copy as i32)
                .unwrap();
        }
        Ok(read)
    }

    /// Write to a open file.
    pub async fn write(&mut self, file: RawFile, buffer: &[u8]) -> Result<(), DeviceError<D::E>> {
        // #[cfg(feature = "defmt-log")]
        // debug!("write(file={:?}, buffer={:x}", file, buffer);

        // #[cfg(feature = "log")]
        // debug!("write(file={:?}, buffer={:x?}", file, buffer);

        // Clone this so we can touch our other structures. Need to ensure we
        // write it back at the end.
        let file_idx = self.get_file_by_id(file)?;
        let volume_idx = self.get_volume_by_id(self.open_files[file_idx].volume_id)?;

        if self.open_files[file_idx].mode == Mode::ReadOnly {
            // TODO replace error
            return Err(DeviceError::ReadOnly);
        }

        self.open_files[file_idx].dirty = true;

        if self.open_files[file_idx].entry.cluster.0 < RESERVED_ENTRIES {
            // file doesn't have a valid allocated cluster (possible zero-length file), allocate one
            self.open_files[file_idx].entry.cluster =
                match self.open_volumes[volume_idx].volume_type {
                    VolumeType::Fat(ref mut fat) => {
                        fat.alloc_cluster(&mut self.block_device, None, false)
                            .await?
                    }
                };
            // debug!(
            // "Alloc first cluster {:?}",
            // self.open_files[file_idx].entry.cluster
            // );
        }

        // Clone this so we can touch our other structures.
        let volume_idx = self.get_volume_by_id(self.open_files[file_idx].volume_id)?;

        if (self.open_files[file_idx].current_cluster.1) < self.open_files[file_idx].entry.cluster {
            // debug!("Rewinding to start");
            self.open_files[file_idx].current_cluster =
                (0, self.open_files[file_idx].entry.cluster);
        }
        // TODO replace error
        let bytes_until_max =
            usize::try_from(MAX_FILE_SIZE - self.open_files[file_idx].current_offset)
                .map_err(|_| DeviceError::ConversionError)?;
        let bytes_to_write = core::cmp::min(buffer.len(), bytes_until_max);
        let mut written = 0;

        while written < bytes_to_write {
            let mut current_cluster = self.open_files[file_idx].current_cluster;
            // debug!(
            //     "Have written bytes {}/{}, finding cluster {:?}",
            //     written, bytes_to_write, current_cluster
            // );
            let current_offset = self.open_files[file_idx].current_offset;
            let (block_idx, block_offset, block_avail) = match self
                .find_data_on_disk(volume_idx, &mut current_cluster, current_offset)
                .await
            {
                Ok(vars) => {
                    // debug!(
                    //     "Found block_idx={:?}, block_offset={:?}, block_avail={}",
                    //     vars.0, vars.1, vars.2
                    // );
                    vars
                }
                Err(DeviceError::EndOfFile) => {
                    // debug!("Extending file");
                    match self.open_volumes[volume_idx].volume_type {
                        VolumeType::Fat(ref mut fat) => {
                            if fat
                                .alloc_cluster(
                                    &mut self.block_device,
                                    Some(current_cluster.1),
                                    false,
                                )
                                .await
                                .is_err()
                            {
                                return Err(DeviceError::DiskFull);
                            }
                            // debug!("Allocated new FAT cluster, finding offsets...");

                            // debug!("New offset {:?}", new_offset);
                            self.find_data_on_disk(
                                volume_idx,
                                &mut current_cluster,
                                self.open_files[file_idx].current_offset,
                            )
                            .await?
                        }
                    }
                }
                Err(e) => return Err(e),
            };
            let mut blocks = [D::B::new()];
            let to_copy = core::cmp::min(block_avail, bytes_to_write - written);
            if block_offset != 0 {
                // debug!("Partial block write");
                self.block_device.read(&mut blocks, block_idx).await?;
            }
            let block = &mut blocks[0];
            block.content_mut()[block_offset..block_offset + to_copy]
                .copy_from_slice(&buffer[written..written + to_copy]);
            // debug!("Writing block {:?}", block_idx);
            self.block_device.write(&blocks, block_idx).await?;
            written += to_copy;
            self.open_files[file_idx].current_cluster = current_cluster;

            let to_copy = to_copy as u32;
            let new_offset = self.open_files[file_idx].current_offset + to_copy;
            if new_offset > self.open_files[file_idx].entry.size {
                // We made it longer
                self.open_files[file_idx].update_length(new_offset);
            }
            self.open_files[file_idx]
                .seek_from_start(new_offset)
                .unwrap();
            // Entry update deferred to file close, for performance.
        }
        self.open_files[file_idx].entry.attributes.set_archive(true);
        self.open_files[file_idx].entry.mtime = self.time_source.get_timestamp();
        Ok(())
    }

    /// Close a file with the given raw file handle.
    pub async fn close_file(&mut self, file: RawFile) -> Result<(), DeviceError<D::E>> {
        let flush_result = self.flush_file(file).await;
        let file_idx = self.get_file_by_id(file)?;
        self.open_files.swap_remove(file_idx);
        flush_result
    }

    /// Flush (update the entry) for a file with the given raw file handle.
    pub async fn flush_file(&mut self, file: RawFile) -> Result<(), DeviceError<D::E>> {
        let file_info = self
            .open_files
            .iter()
            .find(|info| info.file_id == file)
            .ok_or(DeviceError::BadHandle)?;

        if file_info.dirty {
            let volume_idx = self.get_volume_by_id(file_info.volume_id)?;
            match self.open_volumes[volume_idx].volume_type {
                VolumeType::Fat(ref mut fat) => {
                    // debug!("Updating FAT info sector");
                    fat.update_info_sector(&mut self.block_device).await?;
                    // debug!("Updating dir entry {:?}", file_info.entry);
                    if file_info.entry.size != 0 {
                        // If you have a length, you must have a cluster
                        assert!(file_info.entry.cluster.0 != 0);
                    }
                    fat.write_entry_to_disk(&mut self.block_device, &file_info.entry)
                        .await?;
                }
            };
        }
        Ok(())
    }

    /// Check if any files or folders are open.
    pub fn has_open_handles(&self) -> bool {
        !(self.open_dirs.is_empty() || self.open_files.is_empty())
    }

    /// Consume self and return BlockDevice and TimeSource
    pub fn free(self) -> (D, T) {
        (self.block_device, self.time_source)
    }

    /// Check if a file is at End Of File.
    pub fn file_eof(&self, file: RawFile) -> Result<bool, DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        Ok(self.open_files[file_idx].eof())
    }

    /// Seek a file with an offset from the start of the file.
    pub fn file_seek_from_start(
        &mut self,
        file: RawFile,
        offset: u32,
    ) -> Result<(), DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        self.open_files[file_idx]
            .seek_from_start(offset)
            .map_err(|_| DeviceError::InvalidOffset)?;
        Ok(())
    }

    /// Seek a file with an offset from the current position.
    pub fn file_seek_from_current(
        &mut self,
        file: RawFile,
        offset: i32,
    ) -> Result<(), DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        self.open_files[file_idx]
            .seek_from_current(offset)
            .map_err(|_| DeviceError::InvalidOffset)?;
        Ok(())
    }

    /// Seek a file with an offset back from the end of the file.
    pub fn file_seek_from_end(
        &mut self,
        file: RawFile,
        offset: u32,
    ) -> Result<(), DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        self.open_files[file_idx]
            .seek_from_end(offset)
            .map_err(|_| DeviceError::InvalidOffset)?;
        Ok(())
    }

    /// Get the length of a file
    pub fn file_length(&self, file: RawFile) -> Result<u32, DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        Ok(self.open_files[file_idx].length())
    }

    /// Get the current offset of a file
    pub fn file_offset(&self, file: RawFile) -> Result<u32, DeviceError<D::E>> {
        let file_idx = self.get_file_by_id(file)?;
        Ok(self.open_files[file_idx].current_offset)
    }

    /// Create a directory in a given directory.
    pub async fn make_dir_in_dir<N>(
        &mut self,
        directory: RawDirectory,
        name: N,
    ) -> Result<(), DeviceError<D::E>>
    where
        N: ToShortFileName,
    {
        // This check is load-bearing - we do an unchecked push later.
        if self.open_dirs.is_full() {
            return Err(DeviceError::TooManyOpenDirs);
        }

        let parent_directory_idx = self.get_dir_by_id(directory)?;
        let parent_directory_info = &self.open_dirs[parent_directory_idx];
        let volume_id = self.open_dirs[parent_directory_idx].volume_id;
        let volume_idx = self.get_volume_by_id(volume_id)?;
        let volume_info = &self.open_volumes[volume_idx];
        let sfn = name
            .to_short_filename()
            .map_err(DeviceError::FilenameError)?;

        // debug!("Creating directory '{}'", sfn);
        // debug!(
        //     "Parent dir is in cluster {:?}",
        //     parent_directory_info.cluster
        // );

        // Does an entry exist with this name?
        let maybe_dir_entry = match &volume_info.volume_type {
            VolumeType::Fat(fat) => {
                fat.find_directory_entry(&mut self.block_device, parent_directory_info, &sfn)
                    .await
            }
        };

        match maybe_dir_entry {
            Ok(entry) if entry.attributes.is_directory() => {
                return Err(DeviceError::DirAlreadyExists);
            }
            Ok(_) => {
                return Err(DeviceError::FileAlreadyExists);
            }
            Err(DeviceError::NotFound) => {
                // perfect, let's make it
            }
            Err(e) => {
                // Some other error - tell them about it
                return Err(e);
            }
        };

        let att = Attributes::create_from_fat(Attributes::DIRECTORY);

        // Need mutable access for this
        match &mut self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => {
                // debug!("Making dir entry");
                let mut new_dir_entry_in_parent = fat
                    .write_new_directory_entry(
                        &mut self.block_device,
                        &self.time_source,
                        parent_directory_info.cluster,
                        sfn,
                        att,
                    )
                    .await?;
                if new_dir_entry_in_parent.cluster == ClusterId::EMPTY {
                    new_dir_entry_in_parent.cluster = fat
                        .alloc_cluster(&mut self.block_device, None, false)
                        .await?;
                    // update the parent dir with the cluster of the new dir
                    fat.write_entry_to_disk(&mut self.block_device, &new_dir_entry_in_parent)
                        .await?;
                }
                let new_dir_start_block = fat.cluster_to_block(new_dir_entry_in_parent.cluster);
                // debug!("Made new dir entry {:?}", new_dir_entry_in_parent);
                let now = self.time_source.get_timestamp();
                let fat_type = fat.get_fat_type();
                // A blank block
                let mut blocks = [D::B::new()];
                // make the "." entry
                let dot_entry_in_child = DirEntry {
                    name: ShortFileName::this_dir(),
                    mtime: now,
                    ctime: now,
                    attributes: att,
                    // point at ourselves
                    cluster: new_dir_entry_in_parent.cluster,
                    size: 0,
                    entry_block: new_dir_start_block,
                    entry_offset: 0,
                };
                // debug!("New dir has {:?}", dot_entry_in_child);
                let mut offset = 0;
                blocks[0].content_mut()[offset..offset + OnDiskDirEntry::LEN]
                    .copy_from_slice(&dot_entry_in_child.serialize(fat_type)[..]);
                offset += OnDiskDirEntry::LEN;
                // make the ".." entry
                let dot_dot_entry_in_child = DirEntry {
                    name: ShortFileName::parent_dir(),
                    mtime: now,
                    ctime: now,
                    attributes: att,
                    // point at our parent
                    cluster: match fat_type {
                        FatType::Fat16 => {
                            // On FAT16, indicate parent is root using Cluster(0)
                            if parent_directory_info.cluster == ClusterId::ROOT_DIR {
                                ClusterId::EMPTY
                            } else {
                                parent_directory_info.cluster
                            }
                        }
                        FatType::Fat32 => parent_directory_info.cluster,
                    },
                    size: 0,
                    entry_block: new_dir_start_block,
                    entry_offset: OnDiskDirEntry::LEN_U32,
                };
                // debug!("New dir has {:?}", dot_dot_entry_in_child);
                blocks[0].content_mut()[offset..offset + OnDiskDirEntry::LEN]
                    .copy_from_slice(&dot_dot_entry_in_child.serialize(fat_type)[..]);

                self.block_device
                    .write(&blocks, new_dir_start_block)
                    .await?;

                // Now zero the rest of the cluster
                for b in blocks[0].content_mut().iter_mut() {
                    *b = 0;
                }
                for block in new_dir_start_block
                    .range(BlockCount(u32::from(fat.blocks_per_cluster)))
                    .skip(1)
                {
                    self.block_device.write(&blocks, block).await?;
                }
            }
        };

        Ok(())
    }

    fn get_volume_by_id(&self, volume: RawVolume) -> Result<usize, DeviceError<D::E>> {
        for (idx, v) in self.open_volumes.iter().enumerate() {
            if v.volume_id == volume {
                return Ok(idx);
            }
        }
        Err(DeviceError::BadHandle)
    }

    fn get_dir_by_id(&self, directory: RawDirectory) -> Result<usize, DeviceError<D::E>> {
        for (idx, d) in self.open_dirs.iter().enumerate() {
            if d.directory_id == directory {
                return Ok(idx);
            }
        }
        Err(DeviceError::BadHandle)
    }

    fn get_file_by_id(&self, file: RawFile) -> Result<usize, DeviceError<D::E>> {
        for (idx, f) in self.open_files.iter().enumerate() {
            if f.file_id == file {
                return Ok(idx);
            }
        }
        Err(DeviceError::BadHandle)
    }

    /// This function turns `desired_offset` into an appropriate block to be
    /// read. It either calculates this based on the start of the file, or
    /// from the last cluster we read - whichever is better.
    async fn find_data_on_disk(
        &mut self,
        volume_idx: usize,
        start: &mut (u32, ClusterId),
        desired_offset: u32,
    ) -> Result<(BlockIdx, usize, usize), DeviceError<D::E>> {
        let bytes_per_cluster = match &self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => fat.bytes_per_cluster(),
        };
        // How many clusters forward do we need to go?
        let offset_from_cluster = desired_offset - start.0;
        let num_clusters = offset_from_cluster / bytes_per_cluster;
        let mut block_cache = BlockCache::empty();
        for _ in 0..num_clusters {
            start.1 = match &self.open_volumes[volume_idx].volume_type {
                VolumeType::Fat(fat) => {
                    fat.next_cluster(&mut self.block_device, start.1, &mut block_cache)
                        .await?
                }
            };
            start.0 += bytes_per_cluster;
        }
        // How many blocks in are we?
        let offset_from_cluster = desired_offset - start.0;
        assert!(offset_from_cluster < bytes_per_cluster);
        let num_blocks = BlockCount(offset_from_cluster / BLOCK_LEN);
        let block_idx = match &self.open_volumes[volume_idx].volume_type {
            VolumeType::Fat(fat) => fat.cluster_to_block(start.1),
        } + num_blocks;
        let block_offset = (desired_offset % BLOCK_LEN) as usize;
        let available = BLOCK_LEN as usize - block_offset;
        Ok((block_idx, block_offset, available))
    }
}

/// Transform mode variants (ReadWriteCreate_Or_Append) to simple modes ReadWriteAppend or
/// ReadWriteCreate
fn solve_mode_variant(mode: Mode, dir_entry_is_some: bool) -> Mode {
    let mut mode = mode;
    if mode == Mode::ReadWriteCreateOrAppend {
        if dir_entry_is_some {
            mode = Mode::ReadWriteAppend;
        } else {
            mode = Mode::ReadWriteCreate;
        }
    } else if mode == Mode::ReadWriteCreateOrTruncate {
        if dir_entry_is_some {
            mode = Mode::ReadWriteTruncate;
        } else {
            mode = Mode::ReadWriteCreate;
        }
    }
    mode
}

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

#[cfg(test)]
mod tests {
    use crate::{fat::{info::{Fat32Info, FatSpecificInfo}, volume::VolumeName}, filesystem::timestamp::Timestamp};
    use hex_literal::hex;
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Block {
        pub inner: [u8; BLOCK_LEN as usize]
    }
    
    impl BlockTrait for Block {
        fn new() -> Self {
            Self {
                inner: [0u8; BLOCK_LEN as usize],
            }
        }
    
        fn content_mut(&mut self) -> &mut [u8; BLOCK_LEN as usize] {
            &mut self.inner
        }
    
        fn content(&self) -> &[u8; BLOCK_LEN as usize] {
            &self.inner
        }
    }
    

    struct DummyBlockDevice{
        blocks: [Block; 32]
    }

    struct Clock;

    #[derive(Debug)]
    enum Error {}

    impl TimeSource for Clock {
        fn get_timestamp(&self) -> Timestamp {
            // TODO: Return actual time
            Timestamp {
                year_since_1970: 0,
                zero_indexed_month: 0,
                zero_indexed_day: 0,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }
        }
    }

    impl DummyBlockDevice{
        pub fn new() -> Self {
            let mut blocks = [Block{inner: [0u8; BLOCK_LEN as usize]}; 32];
            blocks[0] = Block {
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
            };
            blocks[1] = Block {
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
            };
            blocks[2] = Block {
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
            };
            
            Self{
                blocks 
            }
        }
    }

    impl BlockDevice for DummyBlockDevice {
        type B = Block;
        
        type E = Error;
        /// Read one or more blocks, starting at the given block index.
        async fn read(
            &mut self,
            blocks: &mut [Self::B],
            start_block_idx: BlockIdx,
        ) -> Result<(), DeviceError<Self::E>> {
            // Actual blocks taken from an SD card, except I've changed the start and length of partition 0.
            println!(
                "Reading block {} to {}",
                start_block_idx.0,
                start_block_idx.0 as usize + blocks.len()
            );
            for (idx, block) in blocks.iter_mut().enumerate() {
                let block_idx = start_block_idx.0 as usize + idx;
                if block_idx < self.blocks.len() {
                    *block = self.blocks[block_idx].clone();
                } else {
                    return Err(DeviceError::Unsupported);
                }
            }
            Ok(())
        }
        
        async fn write(
            &mut self,
            blocks: &[Self::B],
            start_block_idx: BlockIdx,
        ) -> Result<(), DeviceError<Self::E>> {
            println!(
                "Writing block {} to {}",
                start_block_idx.0,
                start_block_idx.0 as usize + blocks.len()
            );
            for (idx, block) in blocks.iter().enumerate() {
                let block_idx = start_block_idx.0 as usize + idx;
                if block_idx < self.blocks.len() {
                    self.blocks[block_idx] = *block;
                } else {
                    return Err(DeviceError::Unsupported);
                }
            }
            Ok(())
        }
        
        async fn num_blocks(&self) -> Result<BlockCount, DeviceError<Self::E>> {
            Ok(BlockCount(self.blocks.len() as u32))
        }

        
    }

    #[tokio::test]
    async fn partition0() {
        let mut c: VolumeManager<DummyBlockDevice, Clock, 2, 2> =
            VolumeManager::new_with_limits(DummyBlockDevice::new(), Clock, 0xAA00_0000);

        let v = c.open_raw_volume(VolumeIdx(0)).await.unwrap();
        assert_eq!(v, c.open_volumes[0].volume_id);
        assert_eq!(
            &c.open_volumes[0],
            &VolumeInfo {
                volume_id: RawVolume(SearchId(0xAA00_0000)),
                idx: VolumeIdx(0),
                volume_type: VolumeType::Fat(FatVolume {
                    lba_start: BlockIdx(1),
                    num_blocks: BlockCount(0x0011_2233),
                    blocks_per_cluster: 8,
                    first_data_block: BlockCount(15136),
                    fat_start: BlockCount(32),
                    name: VolumeName::new(*b"Pictures   "),
                    free_clusters_count: None,
                    next_free_cluster: None,
                    cluster_count: 965_788,
                    fat_specific_info: FatSpecificInfo::Fat32(Fat32Info {
                        first_root_dir_cluster: ClusterId(2),
                        info_location: BlockIdx(1) + BlockCount(1),
                    })
                })
            }
        );
    }
}
