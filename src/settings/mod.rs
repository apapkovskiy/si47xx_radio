//! Settings module using Embassy EKV for persistent key-value storage.
//!
//! This module provides a type-safe interface for storing and retrieving
//! settings values (integers, booleans, and strings) with default fallbacks.
//!
//! ## Usage Example
//! ```ignore
//! // Initialize the settings database
//! Settings::init(0x40000, 64 * 1024).await.unwrap();
//!
//! ```
use crate::boards::hal::*;
use crate::storage::Storage;
use ekv::flash;
use ekv::flash::PageID;
use ekv::{CommitError, Config, Database, FormatError, ReadError, WriteError, config};
use embassy_embedded_hal::flash::partition::Partition;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::once_lock::OnceLock;
use embedded_storage_async::nor_flash::{ErrorType, NorFlash, ReadNorFlash};
use heapless::{String, Vec};
use linkme::distributed_slice;
use log::warn;

pub mod option;
use option::OptionString;

#[distributed_slice]
pub static OPTIONS: [&'static OptionString<64>];

/// Maximum string length for stored values
pub const MAX_STRING_LENGTH: usize = 64;
const WRITE_BUFFER_SIZE: usize = 64;

/// Type alias for fixed-size strings
pub type SettingsString = String<MAX_STRING_LENGTH>;

type FlashPartition = Partition<'static, CriticalSectionRawMutex, HalQspi>;

struct Flash {
    partition: FlashPartition,
}

type SettingsDb = Database<Flash, CriticalSectionRawMutex>;

static DB: OnceLock<SettingsDb> = OnceLock::new();

#[derive(Debug, PartialEq, Eq)]
pub enum SettingsError<E = <FlashPartition as ErrorType>::Error> {
    /// Settings database is already initialized
    AlreadyInitialized,
    /// Error reading from flash
    ReadError(ReadError<E>),
    /// Error writing to flash
    WriteError(WriteError<E>),
    /// Capacity error (e.g. value too large)
    CapacityError,
    /// Serialization error
    SerializationError,
    /// Formatting error
    FormatError(FormatError<E>),
    /// Erase error
    EraseError(E),
    /// Commit error
    CommitError(CommitError<E>),
}

impl flash::Flash for Flash {
    type Error = <FlashPartition as ErrorType>::Error;
    fn page_count(&self) -> usize {
        self.partition.capacity() / config::PAGE_SIZE
    }

    async fn read(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        let address = page_id.index() * config::PAGE_SIZE + offset;
        self.partition.read(address as u32, data).await
    }

    async fn write(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let base_address = page_id.index() * config::PAGE_SIZE + offset;
        // Some Flash drivers cannot write directly from code/flash,
        // so copy each chunk through a small RAM buffer first.
        let mut write_buffer = [0u8; WRITE_BUFFER_SIZE];

        for (chunk_index, chunk) in data.chunks(WRITE_BUFFER_SIZE).enumerate() {
            write_buffer[..chunk.len()].copy_from_slice(chunk);
            let address = base_address + chunk_index * WRITE_BUFFER_SIZE;
            self.partition
                .write(address as u32, &write_buffer[..chunk.len()])
                .await?;
        }

        Ok(())
    }

    async fn erase(&mut self, page_id: PageID) -> Result<(), Self::Error> {
        let from = page_id.index() * config::PAGE_SIZE;
        let to = from + config::PAGE_SIZE;
        self.partition.erase(from as u32, to as u32).await
    }
}

pub struct Settings;

impl Settings {
    #[allow(clippy::large_stack_frames)]
    fn init_db(offset: u32, size: u32) -> SettingsDb {
        let partition = Storage::partition(offset, size);
        let config = Config::default();
        Database::new(Flash { partition }, config)
    }
    /// Initialize the settings database with a flash partition.
    ///
    /// # Arguments
    /// * `offset` - Starting offset in flash memory
    /// * `size` - Size of the partition
    ///
    /// # Returns
    /// * `Ok(())` if initialization succeeds
    /// * `Err(SettingsError)` if already initialized or other error
    #[allow(clippy::large_stack_frames)]
    pub async fn init(offset: u32, size: u32) -> Result<(), SettingsError> {
        if DB.is_set() {
            return Err(SettingsError::AlreadyInitialized);
        }
        let db = DB.get_or_init(|| Settings::init_db(offset, size));
        // Checking if the database was formatted, if not, format it
        let ret = db.mount().await;
        if ret.is_err() {
            warn!("Database mount failed, attempting to format flash");
            db.lock_flash()
                .await
                .partition
                .erase(0, size)
                .await
                .map_err(SettingsError::EraseError)?;
            db.format().await.map_err(SettingsError::FormatError)?;
        }
        Ok(())
    }

    pub async fn load() -> Result<(), SettingsError> {
        let db = DB.get().await;
        let rtx = db.read_transaction().await;
        for option in OPTIONS {
            let mut buffer = Vec::<u8, MAX_STRING_LENGTH>::new();
            buffer
                .resize_default(MAX_STRING_LENGTH)
                .map_err(|_| SettingsError::CapacityError)?;
            let len = match rtx.read(option.get_key().as_bytes(), &mut buffer).await {
                Ok(len) => len,
                Err(ReadError::KeyNotFound) => {
                    // Key not found, the default value will be used
                    continue;
                }
                Err(e) => return Err(SettingsError::ReadError(e)),
            };
            let mut str = option.str.write().await;
            *str =
                SettingsString::from_utf8(buffer).map_err(|_| SettingsError::SerializationError)?;
            str.truncate(len);
        }
        Ok(())
    }

    #[allow(clippy::large_stack_frames)]
    pub async fn save() -> Result<(), SettingsError> {
        let db = DB.get().await;
        let mut wtx = db.write_transaction().await;
        let mut options: Vec<_, MAX_STRING_LENGTH> = OPTIONS.iter().copied().collect();
        options.sort_unstable_by_key(|o| o.get_key());
        for option in options {
            let val = option.str.read().await;
            wtx.write(option.get_key().as_bytes(), val.as_bytes())
                .await
                .map_err(SettingsError::WriteError)?;
        }
        wtx.commit().await.map_err(SettingsError::CommitError)?;
        Ok(())
    }

    /// Delete a setting by key.
    ///
    /// # Arguments
    /// * `key` - The setting key to delete
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(SettingsError)` on error
    #[allow(dead_code)]
    pub async fn delete(key: &str) -> Result<(), SettingsError> {
        let db = DB.get().await;
        let mut wtx = db.write_transaction().await;

        wtx.delete(key.as_bytes())
            .await
            .map_err(SettingsError::WriteError)?;
        Ok(())
    }
}
