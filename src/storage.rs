//! Async QSPI-backed flash storage with partition helpers.
//!
//! This module provides an asynchronous interface for flash storage using
//! HAL abstraction types. It wraps the QSPI driver in an async mutex,
//! allowing multiple tasks to share a single driver instance safely.
//!
//! The module exposes `embassy_embedded_hal::flash::partition::Partition`,
//! enabling users to create logical partitions on the flash memory. Each
//! partition implements the `embedded_storage_async` NOR flash traits,
//! facilitating read and write operations in an asynchronous context.
//!
//! ## Typical Usage
//! ```ignore
//! let storage = storage::Storage::init(qspi).unwrap();
//! let mut user_part = storage.partition(0, 256 * 1024).unwrap();
//! user_part.write(0, &[0xAA]).await.unwrap();
//! ```

use crate::boards::hal::*;
use embassy_embedded_hal::flash::partition::Partition;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::once_lock::OnceLock;
type FlashMutex = Mutex<CriticalSectionRawMutex, HalQspi>;
type FlashPartition<'a> = Partition<'a, CriticalSectionRawMutex, HalQspi>;

pub struct Storage(FlashMutex);
static FLASH_MUTEX: OnceLock<Storage> = OnceLock::new();

/// Handle for creating async flash partitions.
impl Storage {
    /// Store the QSPI driver inside a global mutex and return a handle.
    fn init() -> Self {
        let q = hal_qspi_create();
        Storage(FlashMutex::new(q))
    }

    /// Create a logical partition at `offset` with `size`.
    pub fn partition(offset: u32, size: u32) -> FlashPartition<'static> {
        Partition::new(&FLASH_MUTEX.get_or_init(Storage::init).0, offset, size)
    }
}
