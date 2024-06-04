//! Volume related tests

use fs::{filesystem::files::Mode, volume_mgr::{VolumeIdx, VolumeManager}, DeviceError};

mod utils;

#[tokio::test]
async fn open_all_volumes() {
    let time_source = utils::make_time_source();
    let disk = utils::make_block_device(utils::DISK_SOURCE).unwrap();
    let mut volume_mgr: VolumeManager<
        utils::RamDisk<Vec<u8>>,
        utils::TestTimeSource,
        4,
        4,
        2,
    > = VolumeManager::new_with_limits(disk, time_source, 0x1000_0000);

    // Open Volume 0
    let fat16_volume = volume_mgr
        .open_raw_volume(VolumeIdx(0)).await
        .expect("open volume 0");

    // Fail to Open Volume 0 again
    assert!(matches!(
        volume_mgr.open_raw_volume(VolumeIdx(0)).await,
        Err(DeviceError::VolumeAlreadyOpen)
    ));

    volume_mgr.close_volume(fat16_volume).expect("close fat16");

    // Open Volume 1
    let fat32_volume = volume_mgr
        .open_raw_volume(VolumeIdx(1)).await
        .expect("open volume 1");

    // Fail to Volume 1 again
    assert!(matches!(
        volume_mgr.open_raw_volume(VolumeIdx(1)).await,
        Err(DeviceError::VolumeAlreadyOpen)
    ));

    // Open Volume 0 again
    let fat16_volume = volume_mgr
        .open_raw_volume(VolumeIdx(0)).await
        .expect("open volume 0");

    // Open any volume - too many volumes (0 and 1 are open)
    assert!(matches!(
        volume_mgr.open_raw_volume(VolumeIdx(0)).await,
        Err(DeviceError::TooManyOpenVolumes)
    ));

    volume_mgr.close_volume(fat16_volume).expect("close fat16");
    volume_mgr.close_volume(fat32_volume).expect("close fat32");

    // This isn't a valid volume
    assert!(matches!(
        volume_mgr.open_raw_volume(VolumeIdx(2)).await,
        Err(DeviceError::FormatError(_e))
    ));

    // This isn't a valid volume
    assert!(matches!(
        volume_mgr.open_raw_volume(VolumeIdx(9)).await,
        Err(DeviceError::NoSuchVolume)
    ));

    let _root_dir = volume_mgr.open_root_dir(fat32_volume).expect("Open dir");

    assert!(matches!(
        volume_mgr.close_volume(fat32_volume),
        Err(DeviceError::VolumeStillInUse)
    ));
}

#[tokio::test]
async fn close_volume_too_early() {
    let time_source = utils::make_time_source();
    let disk = utils::make_block_device(utils::DISK_SOURCE).unwrap();
    let mut volume_mgr = VolumeManager::new(disk, time_source);

    let volume = volume_mgr
        .open_raw_volume(VolumeIdx(0)).await
        .expect("open volume 0");
    let root_dir = volume_mgr.open_root_dir(volume).expect("open root dir");

    // Dir open
    assert!(matches!(
        volume_mgr.close_volume(volume),
        Err(DeviceError::VolumeStillInUse)
    ));

    let _test_file = volume_mgr
        .open_file_in_dir(root_dir, "64MB.DAT", Mode::ReadOnly).await
        .expect("open test file");

    volume_mgr.close_dir(root_dir).unwrap();

    // File open, not dir open
    assert!(matches!(
        volume_mgr.close_volume(volume),
        Err(DeviceError::VolumeStillInUse)
    ));
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
