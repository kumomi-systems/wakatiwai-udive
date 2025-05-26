use core::ffi::c_void;
use core::fmt::Display;

use alloc::string::String;
use crate::*;

/// Input arguments for a boot driver.
pub struct BootDriverArgs<'a> {
  /// A byte vector containing the file to boot.
  pub img: Vec<u8>,
  /// Command line options to use in booting.
  pub cmdline: &'a str,
}

impl Display for BootDriverArgs<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
"cmdline: {:?}",
    self.cmdline
    )
  }
}

impl BootDriver {
  /// Prints the name of this boot driver.
  /// 
  /// Usually, this will just be the name of the OS to boot (e.g. Linux) or
  /// a format to boot (e.g. ELF).
  pub fn name(&self) -> String {
    self.0.name()
  }

  /// Loads this boot driver.
  pub fn load(&mut self) -> Status {
    self.0.load()
  }

  /// Unloads this boot driver.
  pub fn unload(&mut self) -> Status {
    self.0.unload()
  }

  /// Invokes this boot driver.
  /// 
  /// # Arguments
  /// 
  /// - `args` (`&mut BootDriverArgs`) - The arguments to invoke the driver
  ///   with.
  /// 
  /// # Returns
  /// 
  /// - `None` on a successful invokation and execution of the boot driver.
  /// - `Some(Ok(Status))` on a successful invokation but failed execution of
  ///   the boot driver.
  /// - `Some(Err(Status))` on a failed invokation of the boot driver.
  pub fn invoke(&mut self, args: &mut BootDriverArgs) -> Option<Result<Status, Status>> {
    let mut dio = DriverIO {
      inptr:  args as *mut BootDriverArgs as *mut c_void,
      outptr: core::ptr::null_mut()
    };

    let invoke_status = self.0.invoke(&mut dio, BOOT_DRIVER_IO_MEMTYPE);

    if invoke_status.is_ok_and(|t| t.is_success()) {
      return None;
    }
    
    return Some(invoke_status);
  }
}

#[macro_export]
/// This macro sets up an EFI program to be used as a wakatiwai boot driver.
/// 
/// An entry point `_entry` is defined and will recapture the
/// [`BootDriverArgs`] that the driver was invoked with. It will then start a
/// `main` method (the entry point of the driver, for the purposes of the
/// programmer) with these arguments.
/// 
/// This driver may exit if booting fails, in which case the relevant status
/// code will be returned to the caller, or a SUCCESS may be reported if
/// control is returned to the boot driver.
macro_rules! boot_prelude {
  () => {
    use uefi::Status;

    use springboard::boot::BootDriverArgs;

    #[uefi::entry]
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn  _entry() -> Status {
      uefi::helpers::init().unwrap();

      // Locate driver io struct
      let find_io_mem_status = springboard::driver::find_io_memory(springboard::BOOT_DRIVER_IO_MEMTYPE);
      if find_io_mem_status.is_error() {
        return find_io_mem_status;
      }
      let dio = springboard::io::DriverIO::allocated_driver_io().unwrap();

      let main_status = main(unsafe {
        core::mem::transmute::<*mut core::ffi::c_void, *mut BootDriverArgs>(dio.inptr).as_ref().unwrap()
      });

      if main_status.is_none() {
        return Status::SUCCESS;
      }

      main_status.unwrap()
    }
  };
}