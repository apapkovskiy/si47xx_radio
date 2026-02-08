//! Host TWI/I2C (TWIM) implementation for development and testing.
//!
//! This module provides a simulated I2C/TWI interface for host-based testing.
//! It implements the `embedded-hal-async` I2C traits to allow testing I2C
//! communication logic without actual hardware.

use super::Error;

/// Implements the embedded-hal I2C error trait for the host error type.
///
/// This allows the host error type to be used with embedded-hal async I2C traits.
impl embedded_hal_async::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_async::i2c::ErrorKind {
        embedded_hal_async::i2c::ErrorKind::Other
    }
}

/// A simulated I2C/TWI master interface for host-based testing.
///
/// This struct provides a mock implementation of I2C communication that can be
/// used for testing on a development machine without requiring actual hardware.
/// All operations succeed immediately and simulate minimal realistic behavior.
pub struct HostTwim;

/// Implements the error type for the host I2C interface.
impl embedded_hal_async::i2c::ErrorType for HostTwim {
    type Error = Error;
}

/// Implements the async I2C trait for the host TWI interface.
///
/// This provides simulated I2C read, write, and transaction operations.
/// Operations complete successfully with minimal realistic behavior for testing.
impl embedded_hal_async::i2c::I2c for HostTwim {
    /// Simulates reading data from an I2C device.
    ///
    /// Fills the buffer with 0x81 to simulate
    ///  - CTS (Clear To Send) bit (0x80)
    ///  - STCINT (Status Change Interrupt) (0x01)
    ///
    /// being set in a status byte, which is common in radio chip communication.
    ///
    /// # Arguments
    ///
    /// * `_address` - The I2C device address (ignored in simulation).
    /// * `buffer` - A mutable buffer to fill with simulated data.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` to simulate a successful read.
    async fn read(&mut self, _address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        buffer.fill(0x81); // Simulate setting CTS bit in status byte
        Ok(())
    }

    /// Simulates writing data to an I2C device.
    ///
    /// This method accepts data but doesn't perform any actual operation,
    /// simply returning success.
    ///
    /// # Arguments
    ///
    /// * `_address` - The I2C device address (ignored in simulation).
    /// * `_bytes` - The data to write (ignored in simulation).
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` to simulate a successful write.
    async fn write(&mut self, _address: u8, _bytes: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Simulates a combined write-then-read I2C transaction.
    ///
    /// This is commonly used for register reads where you write a register
    /// address and then read the register value. The simulation ignores
    /// both the write data and doesn't fill the read buffer.
    ///
    /// # Arguments
    ///
    /// * `_address` - The I2C device address (ignored in simulation).
    /// * `_bytes` - The data to write (ignored in simulation).
    /// * `_buffer` - A mutable buffer for read data (not modified).
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` to simulate a successful transaction.
    async fn write_read(
        &mut self,
        _address: u8,
        _bytes: &[u8],
        _buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Simulates executing a sequence of I2C operations as a single transaction.
    ///
    /// This allows multiple reads and writes to be performed atomically.
    /// The simulation ignores all operations and returns success.
    ///
    /// # Arguments
    ///
    /// * `_address` - The I2C device address (ignored in simulation).
    /// * `_operations` - A slice of I2C operations to perform (ignored in simulation).
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` to simulate a successful transaction.
    async fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Creates a new host TWI/I2C interface instance.
///
/// This factory function instantiates a `HostTwim` for use in host-based
/// testing scenarios.
///
/// # Returns
///
/// A new `HostTwim` instance ready for simulated I2C operations.
///
/// # Examples
///
/// ```ignore
/// let twim = hal_twi_create();
/// // Use twim with embedded-hal-async I2C traits
/// ```
pub fn hal_twi_create() -> HostTwim {
    HostTwim {}
}
