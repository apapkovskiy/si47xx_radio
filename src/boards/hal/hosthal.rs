//! Hardware abstraction layer for host builds.
//! This module provides mock implementations of HAL types for running
//! the SI47xx radio application in a host environment (e.g., desktop).
//! Since actual hardware peripherals are not available, these implementations
//! simulate the necessary interfaces for testing and development.
mod host_uart;
pub use host_uart::hal_uart_create;
use host_uart::{HostUartRx, HostUartTxBlocking};

mod host_twim;
use host_twim::HostTwim;
pub use host_twim::hal_twi_create;

mod host_digital;
use host_digital::HostOutputPin;

#[derive(Debug)]
pub enum Error {
    FakeError,
}

pub type HalUartTx = HostUartTxBlocking;
pub type HalUartRx = HostUartRx;
pub type HalUartError = Error;
pub type HalTwim = HostTwim;
pub type HalOutput = HostOutputPin;

/// Creates and returns a simulated reset pin for the radio.
/// This pin can be used to control the reset line of the SI47xx radio chip
/// in a host environment.
/// # Returns
/// A new `HalOutput` instance representing the reset pin.
pub fn hal_radio_reset_create() -> HalOutput {
    HostOutputPin {}
}

/// Creates and returns a simulated LED output pin.
/// This pin can be used to control an LED indicator
/// in a host environment.
/// # Returns
/// A new `HalOutput` instance representing the LED pin.
pub fn hal_led_create() -> HalOutput {
    HostOutputPin {}
}
