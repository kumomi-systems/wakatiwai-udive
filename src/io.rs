use core::{ffi::c_void, ptr::null_mut};

use uefi::boot::PAGE_SIZE;

use crate::DRIVER_IO;

/// Used to communicate between `wakatiwai` and drivers.
pub struct DriverIO {
  /// A pointer to the arguments for a driver.
  pub inptr:  *mut c_void,
  /// A pointer to the output of a driver.
  pub outptr: *mut c_void
}

impl DriverIO {
  /// Returns the currently allocated [`DriverIO`], or `None` if unset.
  /// 
  /// # Returns
  /// 
  /// - `Some<&'static mut DriverIO>` if an instance exists.
  /// - `None` otherwise.
  pub fn allocated_driver_io() -> Option<&'static mut DriverIO> {
    unsafe {
      if DRIVER_IO.is_none() {
        return None;
      }

      return Some(
        DRIVER_IO.unwrap().as_mut().unwrap()
      )
    }
  }

  /// Returns the number of pages used by a [`DriverIO`] struct.
  pub const fn page_count() -> usize {
    (size_of::<DriverIO>() + PAGE_SIZE - 1) / PAGE_SIZE
  }

  /// Resets a DriverIO instance.
  /// 
  /// The `inptr` and outptr` fields are set to zero.
  pub fn zero(&mut self) {
    self.inptr  = null_mut();
    self.outptr = null_mut();
  }
}