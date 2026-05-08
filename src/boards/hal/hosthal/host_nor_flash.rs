extern crate std;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

use super::Error;
use embedded_storage_async::nor_flash::{
    ErrorType, NorFlash as AsyncNorFlash, NorFlashError, NorFlashErrorKind,
    ReadNorFlash as AsyncReadNorFlash,
};

pub struct HostNorFlash;

const FILE_PATH: &str = "host_flash.bin";

pub fn hal_qspi_create() -> HostNorFlash {
    HostNorFlash
}

impl ErrorType for HostNorFlash {
    type Error = Error;
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}

impl AsyncReadNorFlash for HostNorFlash {
    const READ_SIZE: usize = 4;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let path = Path::new(FILE_PATH);
        let mut file = File::open(path).map_err(|_| Error::OpenError)?;
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|_| Error::SeekError)?;
        file.read_exact(bytes).map_err(|_| Error::ReadError)
    }

    fn capacity(&self) -> usize {
        16 * 1024 * 1024 // 1 MiB
    }
}

impl AsyncNorFlash for HostNorFlash {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;

    async fn erase(&mut self, from: u32, _to: u32) -> Result<(), Self::Error> {
        let path = Path::new(FILE_PATH);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|_| Error::OpenError)?;
        file.seek(std::io::SeekFrom::Start(from as u64))
            .map_err(|_| Error::SeekError)?;
        let erase_data = [0xFF; HostNorFlash::ERASE_SIZE];
        file.write_all(&erase_data).map_err(|_| Error::EraseError)?;
        Ok(())
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let path = Path::new(FILE_PATH);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(path)
            .map_err(|_| Error::OpenError)?;
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|_| Error::SeekError)?;
        file.write_all(bytes).map_err(|_| Error::WriteError)
    }
}
