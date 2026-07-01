#![no_std]
#![no_main]

extern crate alloc;

use littlefs2::driver::Storage;
use esp_storage::{FlashStorage, FlashStorageError};
use littlefs2::consts;
use littlefs2::io::Error;
use littlefs2::io;
use esp_hal::peripherals::FLASH;
use embedded_storage::ReadStorage;
use embedded_storage::nor_flash::NorFlash;

pub struct PbFlashStorage<'a> {
    internal_flash: FlashStorage<'a>,
}

impl<'a> PbFlashStorage<'a> {
    pub fn new(flash: FLASH<'a>) -> Self {
        Self {
            internal_flash: FlashStorage::new(flash),
        }
    }
}

impl Storage for PbFlashStorage<'_> {
    type CACHE_SIZE = consts::U256;
    type LOOKAHEAD_SIZE = consts::U32;

    const READ_SIZE: usize = 16;
    const WRITE_SIZE: usize = 256;
    const BLOCK_SIZE: usize = 4096;
    const BLOCK_COUNT: usize = 256; // 1MB for: usize now

    // Required methods
    fn read(&mut self, off: usize, buf: &mut [u8]) -> io::Result<usize> {
        let new_off: u32 = off.try_into().map_err(|_| {Error::INVALID})?;
        self.internal_flash.read(new_off, buf).map_err(flash_to_littlefs_error)?;
        Ok(0)
    }

    fn write(&mut self, off: usize, data: &[u8]) -> io::Result<usize> {
        let new_off: u32 = off.try_into().map_err(|_| {Error::INVALID})?;
        self.internal_flash.write(new_off, data).map_err(flash_to_littlefs_error)?;
        Ok(0)
    }

    fn erase(&mut self, off: usize, len: usize) -> io::Result<usize> {
        let from: u32 = off.try_into().map_err(|_| {Error::INVALID})?;
        let to: u32 = (len + off).try_into().map_err(|_| {Error::INVALID})?;
        self.internal_flash.erase(from, to).map_err(flash_to_littlefs_error)?;
        Ok(0)
    }
}

fn flash_to_littlefs_error(e: FlashStorageError) -> Error {
    match e {
        FlashStorageError::IoError => Error::IO,
        FlashStorageError::IoTimeout => Error::new(-106).expect("negative number used so should not panic"), // ETIMEDOUT
        FlashStorageError::CantUnlock => Error::new(-37).expect("negative number used so should not panic"), // ENOLCK
        FlashStorageError::NotAligned => Error::new(-41).expect("negative number used so should not panic"), // NOT IMPLEMENTED
        FlashStorageError::OutOfBounds => Error::new(-14).expect("negative number used so should not panic"), // EFAULT
        FlashStorageError::OtherCoreRunning => Error::new(-16).expect("negative number used so should not panic"), // EBUSY
        FlashStorageError::Other(e) => Error::new(e).expect("negative number used so should not panic"),
        _ => Error::new(-1).expect("negative number used so should not panic")
    }
}
