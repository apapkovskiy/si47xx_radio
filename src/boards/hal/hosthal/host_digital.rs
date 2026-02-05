//! Host digital GPIO pin implementation for development and testing.
//!
//! This module provides a simulated digital output pin interface for host-based
//! testing. It implements the `embedded-hal` digital output traits to allow
//! testing GPIO logic without actual hardware.
use embedded_hal::digital::OutputPin;

/// A simulated digital output pin for host-based testing.
///
/// This struct provides a mock implementation of a digital output pin that can
/// be used for testing on a development machine without requiring actual hardware.
/// All operations succeed immediately without performing any real hardware access.
pub struct HostOutputPin;

/// Implements the error type for the host output pin.
///
/// Uses `Infallible` since simulated pin operations cannot fail.
impl embedded_hal::digital::ErrorType for HostOutputPin {
    type Error = core::convert::Infallible;
}

/// Implements the digital output pin trait for the host pin.
///
/// This provides simulated pin control operations. All operations succeed
/// immediately without performing any actual hardware access.
impl OutputPin for HostOutputPin {
    /// Simulates setting the pin to a low state.
    ///
    /// In a real implementation, this would drive the pin to ground (0V).
    /// This simulation does nothing but returns success.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` since the operation cannot fail in simulation.
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Simulates setting the pin to a high state.
    ///
    /// In a real implementation, this would drive the pin to VCC (logic high).
    /// This simulation does nothing but returns success.
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` since the operation cannot fail in simulation.
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
