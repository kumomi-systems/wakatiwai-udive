use uefi::boot::MemoryType;
use uefi::mem::memory_map::MemoryMap;
use uefi::Status;

use crate::io::DriverIO;
use crate::*;

/// Detects if a given memory type has been allocated.
/// 
/// # Arguments
/// 
/// - `memtype` (`MemoryType`) - The memory type in question, either
///   [`BOOT_DRIVER_IO_MEMTYPE`] or [`FSYS_DRIVER_IO_MEMTYPE`].
/// 
/// # Returns
/// 
/// - [`uefi_raw::Status::NOT_FOUND`] if the memory type has not been allocated.
/// - [`uefi_raw::Status::SUCCESS`] if the memory type has been allocated.
/// 
/// # Safety
/// This function is unsafe because it must access a `static mut` variable.
pub unsafe fn find_io_memory(memtype: MemoryType) -> Status {
  // Iterate over the memory map to detect the buffers
  for mement in uefi::boot::memory_map(MemoryType::LOADER_DATA).unwrap().entries() {
    if mement.ty == memtype {
      DRIVER_IO = Some(mement.phys_start as *mut DriverIO);
      return Status::SUCCESS;
    }
  }

  // The buffer wasn't found
  Status::NOT_FOUND
}