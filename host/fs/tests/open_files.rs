//! File opening related tests

use fs::{filesystem::files::Mode, volume_mgr::{VolumeIdx, VolumeManager}, DeviceError};

mod utils;

#[tokio::test]
async fn open_files() {
    let time_source = utils::make_time_source();
    let disk = utils::make_block_device(utils::DISK_SOURCE).unwrap();
    let mut volume_mgr: VolumeManager<utils::RamDisk<Vec<u8>>, utils::TestTimeSource, 4, 2, 1> =
        VolumeManager::new_with_limits(disk, time_source, 0xAA00_0000);
    let volume = volume_mgr
        .open_raw_volume(VolumeIdx(0)).await
        .expect("open volume");
    let root_dir = volume_mgr.open_root_dir(volume).expect("open root dir");

    // Open with string
    let f = volume_mgr
        .open_file_in_dir(root_dir, "README.TXT", Mode::ReadWriteTruncate).await
        .expect("open file");

    assert!(matches!(
        volume_mgr.open_file_in_dir(root_dir, "README.TXT", Mode::ReadOnly).await,
        Err(DeviceError::FileAlreadyOpen)
    ));

    volume_mgr.close_file(f).await.expect("close file");

    // Open with SFN

    let dir_entry = volume_mgr
        .find_directory_entry(root_dir, "README.TXT").await
        .expect("find file");

    let f = volume_mgr
        .open_file_in_dir(root_dir, &dir_entry.name, Mode::ReadWriteCreateOrAppend).await
        .expect("open file with dir entry");

    assert!(matches!(
        volume_mgr.open_file_in_dir(root_dir, &dir_entry.name, Mode::ReadOnly).await,
        Err(DeviceError::FileAlreadyOpen)
    ));

    // Can still spot duplicates even if name given the other way around

    assert!(matches!(
        volume_mgr.open_file_in_dir(root_dir, "README.TXT", Mode::ReadOnly).await,
        Err(DeviceError::FileAlreadyOpen)
    ));

    let f2 = volume_mgr
        .open_file_in_dir(root_dir, "64MB.DAT", Mode::ReadWriteTruncate).await
        .expect("open file");

    // Hit file limit

    assert!(matches!(
        volume_mgr.open_file_in_dir(root_dir, "EMPTY.DAT", Mode::ReadOnly).await,
        Err(DeviceError::TooManyOpenFiles)
    ));

    volume_mgr.close_file(f).await.expect("close file");
    volume_mgr.close_file(f2).await.expect("close file");

    // File not found

    assert!(matches!(
        volume_mgr.open_file_in_dir(root_dir, "README.TXS", Mode::ReadOnly).await,
        Err(DeviceError::NotFound)
    ));

    // Create a new file
    let f3 = volume_mgr
        .open_file_in_dir(root_dir, "NEWFILE.DAT", Mode::ReadWriteCreate).await
        .expect("open file");

    volume_mgr.write(f3, b"12345").await.expect("write to file");
    volume_mgr.write(f3, b"67890").await.expect("write to file");
    volume_mgr.close_file(f3).await.expect("close file");

    // Open our new file
    let f3 = volume_mgr
        .open_file_in_dir(root_dir, "NEWFILE.DAT", Mode::ReadOnly).await
        .expect("open file");
    // Should have 10 bytes in it
    assert_eq!(volume_mgr.file_length(f3).expect("file length"), 10);
    volume_mgr.close_file(f3).await.expect("close file");

    volume_mgr.close_dir(root_dir).expect("close dir");
    volume_mgr.close_volume(volume).expect("close volume");
}

#[tokio::test]
async fn open_non_raw() {
    let time_source = utils::make_time_source();
    let disk = utils::make_block_device(utils::DISK_SOURCE).unwrap();
    let mut volume_mgr: VolumeManager<utils::RamDisk<Vec<u8>>, utils::TestTimeSource, 4, 2, 1> =
        VolumeManager::new_with_limits(disk, time_source, 0xAA00_0000);
    let mut volume = volume_mgr.open_volume(VolumeIdx(0)).await.expect("open volume");
    let mut root_dir = volume.open_root_dir().expect("open root dir");
    let mut f = root_dir
        .open_file_in_dir("README.TXT", Mode::ReadOnly).await
        .expect("open file");

    let mut buffer = [0u8; 512];
    let len = f.read(&mut buffer).await.expect("read from file");
    // See directory listing in utils.rs, to see that README.TXT is 258 bytes long
    assert_eq!(len, 258);
    assert_eq!(f.length(), 258);
    f.seek_from_current(0).unwrap();
    assert_eq!(f.is_eof(), true);
    assert_eq!(f.offset(), 258);
    assert!(matches!(f.seek_from_current(1), Err(DeviceError::InvalidOffset)));
    f.seek_from_current(-258).unwrap();
    assert_eq!(f.is_eof(), false);
    assert_eq!(f.offset(), 0);
    f.seek_from_current(10).unwrap();
    assert_eq!(f.is_eof(), false);
    assert_eq!(f.offset(), 10);
    f.seek_from_end(0).unwrap();
    assert_eq!(f.is_eof(), true);
    assert_eq!(f.offset(), 258);
    assert!(matches!(
        f.seek_from_current(-259),
        Err(DeviceError::InvalidOffset)
    ));
    f.seek_from_start(25).unwrap();
    assert_eq!(f.is_eof(), false);
    assert_eq!(f.offset(), 25);

    drop(f);

    let res = root_dir.open_file_in_dir("README.TXT", Mode::ReadWriteCreate).await;
    if let Ok(_) = res{
        panic!("File must exist");
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
