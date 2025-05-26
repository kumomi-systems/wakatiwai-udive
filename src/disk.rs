use alloc::vec::Vec;
use uefi::proto::media::block::BlockIO;
use uefi::proto::media::disk::DiskIo;
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::{Handle, Status};

/// Manages reading a disk.
/// 
/// Instances of [`DiskReader`] operate as an abstraction of a UEFI `DiskIo`
/// protocol. It allows for raw low-level access to a disk, reading in
/// intervals of bytes, sectors, and blocks.
pub struct DiskReader {
  /// The protocol over which to abstract.
  protocol: ScopedProtocol<DiskIo>,
  /// The offset within the disk to read from.
  /// 
  /// In reading a file system, this will usually be set to the offset of the
  /// partition on which it resides.
  pub abs_offset: u64,
  /// The media ID of the partition.
  pub media_id: u32,
  /// The number of bytes that make up a physical sector on this disk.
  pub sector_size: u32,
  /// The number of bytes that make up a logical block on this disk.
  pub block_size: u32,
  /// The final LBA of this partition.
  pub last_block: u64
}

impl DiskReader {
  /// Creates a new diskreader.
  /// 
  /// # Arguments
  /// 
  /// - `handle` (`&Handle`) - The EFI handle to the partition on which to
  ///   create a disk reader.
  /// - `protocol` (`ScopedProtocol<DiskIo>`) - An instance of the
  ///   `DiskIo` protocol, currently open on the aforementioned handle.
  /// - `abs_offset` (`u64`) - The offset on the disk to read from.
  /// 
  /// # Returns
  /// 
  /// - `DiskReader` - An instance of a [`DiskReader`]
  pub fn new(handle: &Handle, protocol: ScopedProtocol<DiskIo>, abs_offset: u64) -> DiskReader {    
    let media_id: u32;
    let sector_size: u32;
    let block_size: u32;
    let last_block: u64;

    unsafe {
      let block_io_protocol = uefi::boot::open_protocol::<BlockIO>(
        OpenProtocolParams {
          handle: *handle,
          agent: uefi::boot::image_handle(),
          controller: None
        },
        OpenProtocolAttributes::GetProtocol
      ).unwrap();
      
      media_id = block_io_protocol.media().media_id();
      block_size = block_io_protocol.media().block_size();
      if block_io_protocol.media().logical_blocks_per_physical_block() == 0 {
        sector_size = block_size;
      } else { 
        sector_size = block_size / block_io_protocol.media().logical_blocks_per_physical_block();
      }
      last_block = block_io_protocol.media().last_block();
    }

    DiskReader {
      protocol,
      abs_offset,
      media_id,
      sector_size,
      block_size,
      last_block
    }
  }

  /// Reads a number of bytes from the disk at a specified offset.
  /// 
  /// # Arguments
  /// 
  /// - `offset` (`u64`) - The disk offset to read from.
  /// - `count` (`usize`) - The number of bytes to read.
  /// 
  /// # Returns
  /// 
  /// - `Ok(Vec<u8>)` on success, containing the bytes read.
  /// - `Err(Status)` on failure.
  pub fn read_bytes(&self, offset: u64, count: usize) -> Result<Vec<u8>, Status> {
    let mut buffer = alloc::vec![0 as u8; count];
    let status = self.protocol.read_disk(
      self.media_id,
      self.abs_offset + offset,
      &mut buffer
    );
    
    if status.is_err() {
      return Err(status.err().unwrap().status());
    }
    Ok(buffer)
  }

  /// Reads the given sector from the disk.
  /// 
  /// # Arguments
  /// 
  /// - `sector` (`u64`) - The number of the sector to read.
  /// 
  /// # Returns
  /// 
  /// - `Ok(Vec<u8>)` on success, containing the sector's data.
  /// - `Err(Status)` on failure.
  pub fn read_sector(&self, sector: u64) -> Result<Vec<u8>, Status> {
    self.read_bytes(sector * self.sector_size as u64, self.sector_size as usize)
  }

  /// Reads the given number of sectors from the disk.
  /// 
  /// # Arguments
  /// 
  /// - `sector` (`u64`) - The number of the first sector to read.
  /// - `count` (`usize`) - The number of sectors to read.
  /// 
  /// # Returns
  /// 
  /// - `Ok(Vec<u8>)` on success, containing the data of those sectors.
  /// - `Err(Status)` on failure.
  pub fn read_sectors(&self, sector: u64, count: usize) -> Result<Vec<u8>, Status> {
    self.read_bytes(sector * self.sector_size as u64, count * self.sector_size as usize)
  }

  /// Reads the given block from the disk.
  /// 
  /// # Arguments
  /// 
  /// - `lba` (`u64`) - The LBA of the block to read.
  /// 
  /// # Returns
  /// 
  /// - `Ok(Vec<u8>)` on success, containing the block's data.
  /// - `Err(Status)` on failure.
  pub fn read_block(&self, lba: u64) -> Result<Vec<u8>, Status> {
    self.read_bytes(lba * self.block_size as u64, self.block_size as usize)
  }

  /// Reads the given number of blocks from the disk.
  /// 
  /// # Arguments
  /// 
  /// - `block` (`u64`) - The LBA of the first block to read.
  /// - `count` (`usize`) - The number of blocks to read.
  /// 
  /// # Returns
  /// 
  /// - `Ok(Vec<u8>)` on success, containing the data of those blocks.
  /// - `Err(Status)` on failure.
  pub fn read_blocks(&self, lba: u64, count: usize) -> Result<Vec<u8>, Status> {
    self.read_bytes(lba * self.block_size as u64, count * self.block_size as usize)
  }
}