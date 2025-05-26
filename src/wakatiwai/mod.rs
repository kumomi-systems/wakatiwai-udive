use uefi::proto::media::file::{Directory, File, FileAttribute, FileHandle, FileInfo, FileMode};
use uefi::{CString16, Error, Status};

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::ToString;

use crate::*;

/// Open the directory containing the `boot` and `fs` driver subdirectories.
/// 
/// The image file system is opened on the root directory, which in turn opens
/// [`DRIVER_DIRECTORY`].
/// 
/// # Returns
/// - `Ok(uefi::proto::media::file::FileHandle)` on success.
/// - `Err(uefi::result::Error)` on failure.
fn get_driver_dir() -> Result<FileHandle, Error> {
  let mut efifs = uefi::boot::get_image_file_system(uefi::boot::image_handle())?;
  let mut efivol = efifs.open_volume()?;
  efivol.open(
    DRIVER_DIRECTORY,
    FileMode::Read,
    FileAttribute::DIRECTORY
  )
}

/// Open the directory containing boot drivers.
/// 
/// The driver directory is opened via [`get_driver_dir`], and
/// [`BOOT_DRIVER_DIRECTORY`] is thne in turn opened.
/// 
/// # Returns
/// - `Ok(uefi::proto::media::file::FileHandle)` on success.
/// - `Err(uefi::result::Error)` on failure.
fn get_boot_driver_dir() -> Result<FileHandle, Error> {
  get_driver_dir().unwrap().into_directory().unwrap().open(
    BOOT_DRIVER_DIRECTORY,
    FileMode::Read,
    FileAttribute::DIRECTORY
  )
}

/// Open the directory containing file system drivers.
/// 
/// The driver directory is opened via [`get_driver_dir`], and
/// [`FSYS_DRIVER_DIRECTORY`] is thne in turn opened.
/// 
/// # Returns
/// - `Ok(uefi::proto::media::file::FileHandle)` on success.
/// - `Err(uefi::result::Error)` on failure.
fn get_fs_driver_dir() -> Result<FileHandle, Error> {
  get_driver_dir()?.into_directory().unwrap().open(
    FSYS_DRIVER_DIRECTORY,
    FileMode::Read,
    FileAttribute::DIRECTORY
  )
}

/// Validates a driver from its EFI file.
/// 
/// A file handle is checked and deemed valid if it meets the following:
/// - It points to a regular file.
/// - The file has a `.efi` extension.
fn is_valid_driver(handle: &mut FileHandle) -> bool {
  // TODO: Make checks more restrictive
  let driver_info: Box<FileInfo> = handle.get_boxed_info().unwrap();
  
  // Ensure that the driver is a file
  if !driver_info.is_regular_file() {
    return false;
  }
  // Ensure that the driver has the efi extension
  if !driver_info.file_name().to_string().ends_with(".efi") {
    return false;
  }

  false
}

/// Returns all drivers from a directory.
/// 
/// A reference to a UEFI directory is given, and all valid drivers therein are
/// returned.
/// 
/// # Returns
/// - `Ok(Vec<Driver>)` on success.
/// - `Err(uefi::Status)` on failure.
fn get_driver_files_from_dir(directory: &mut Directory) -> Result<Vec<Driver>, Status> {
  let mut drivers: Vec<Driver> = Vec::new();

  loop {
    match directory.read_entry_boxed() {
      Ok(ok) => {
        if ok.is_none() {
          // No more files
          break;
        }

        // Open a handle on the directory member
        let mut file_handle = directory.open(
          ok.unwrap().file_name(),
          FileMode::Read,
          FileAttribute::READ_ONLY
        );
        
        // If unable to read, error
        if file_handle.is_err() {
          return Err(file_handle.err().unwrap().status());
        }

        // Skip if invalid
        if !is_valid_driver(file_handle.as_mut().unwrap()) {
          continue;
        }

        let mut driver_name = CString16::new();
        driver_name.push_str((file_handle.as_mut().unwrap().get_boxed_info().unwrap() as Box<FileInfo>).file_name());

        drivers.push(
          Driver {
            name: driver_name,
            driver_type: None,
            exec_handle: None
          }
        );
      }
      Err(_) => {
        break;
      }
    }
  }

  Ok(drivers)
}

/// Returns all boot drivers.
/// 
/// The boot drivers directory is opened and all valid boot drivers are
/// returned.
fn get_boot_drivers() -> Result<Vec<BootDriver>, Status> {
  let mut ret: Vec<BootDriver> = Vec::new();
  match get_boot_driver_dir() {
    Ok(ok) => {
      for driver in get_driver_files_from_dir(&mut ok.into_directory().unwrap())?.iter() {
        ret.push(
          BootDriver(
            Driver {
              name: driver.name.clone(),
              driver_type: Some(DriverType::BOOT),
              exec_handle: driver.exec_handle
            }
          )
        )
      }

      return Ok(ret);
    }
    Err(err) => {
      return Err(err.status());
    }
  }
}

/// Returns all file system drivers.
/// 
/// The file system drivers directory is opened and all valid file system
/// drivers are returned.
fn get_fs_drivers() -> Result<Vec<FSDriver>, Status> {
  let mut ret: Vec<FSDriver> = Vec::new();
  match get_fs_driver_dir() {
    Ok(ok) => {
      for driver in get_driver_files_from_dir(&mut ok.into_directory().unwrap())?.iter() {
        ret.push(
          FSDriver(
            Driver {
              name: driver.name.clone(),
              driver_type: Some(DriverType::FS),
              exec_handle: driver.exec_handle
            }
          )
        )
      }

      return Ok(ret)
    }
    Err(err) => {
      return Err(err.status());
    }
  }
}

/// Attempts to obtain a specified boot driver.
/// 
/// The boot drivers directory is opened and attempts to return the driver with
/// the given name.
/// 
/// # Returns
/// - `Ok(Some(BootDriver))` - The inner [`BootDriver`] is the requested driver.
/// - `Ok(None)` - The boot driver could not be found.
/// - `Err(Status)` - The boot driver directory could not be opened.
pub fn get_boot_driver(driver_name: &str) -> Result<Option<BootDriver>, Status> {
  let boot_drivers = get_boot_drivers();
  if boot_drivers.is_err() {
    return Err(boot_drivers.err().unwrap());
  }

  for boot_driver in boot_drivers.unwrap() {
    if boot_driver.name() == driver_name {
      return Ok(Some(boot_driver));
    }
  }

  return Ok(None);
}

/// Attempts to obtain a specified file system driver.
/// 
/// The file system drivers directory is opened and attempts to return the 
/// driver with the given name.
/// 
/// # Returns
/// - `Ok(Some(FSDriver))` - The inner [`FSDriver`] is the requested driver.
/// - `Ok(None)` - The file system driver could not be found.
/// - `Err(Status)` - The file system driver directory could not be opened.
pub fn get_fs_driver(driver_name: &str) -> Result<Option<FSDriver>, Status> {
  let fs_drivers = get_fs_drivers();
  if fs_drivers.is_err() {
    return Err(fs_drivers.err().unwrap());
  }

  for fs_driver in fs_drivers.unwrap() {
    if fs_driver.name() == driver_name {
      return Ok(Some(fs_driver));
    }
  }

  return Ok(None);
}