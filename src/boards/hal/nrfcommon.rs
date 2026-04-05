#![cfg(feature = "nrf")]

//! Common HAL types and bindings for nRF-based boards.
//!
//! This module provides shared type definitions and interrupt bindings that are
//! common across all nRF microcontroller-based boards. It includes HAL types for:
//!
//! - QSPI (Quad SPI) flash memory interface
//! - UART for serial communication
//! - TWI/I2C for the SI47xx radio communication
//! - GPIO outputs for control signals
//!
//! # Type Aliases
//!
//! The module exports type aliases for Embassy HAL peripherals to simplify
//! board-specific implementations:
//! - [`HalQspi`]: QSPI flash interface
//! - [`HalUartTx`]: UART serial transmit interface
//! - [`HalUartRx`]: UART serial receive interface
//! - [`HalUartError`]: UART error type
//! - [`HalTwim`]: Two-Wire Interface Master (I2C)
//! - [`HalOutput`]: GPIO output pin

use embassy_nrf::gpio::Output;
use embassy_nrf::peripherals::{QSPI, SERIAL0, SERIAL1};
use embassy_nrf::qspi;
use embassy_nrf::{bind_interrupts, twim, uarte};

// Interrupt bindings for nRF peripherals.
//
// This macro binds interrupt handlers to the respective peripheral drivers
// used by the SI47xx radio application.
bind_interrupts!(pub struct Irqs {
    SERIAL0 => uarte::InterruptHandler<SERIAL0>;
    SERIAL1 => twim::InterruptHandler<SERIAL1>;
    QSPI => qspi::InterruptHandler<QSPI>;
});

/// QSPI flash interface type.
///
/// Used for non-volatile storage of radio settings and configuration.
pub type HalQspi = qspi::Qspi<'static>;

/// UART serial interface type.
///
/// Used for console output, debugging, and user interaction.
pub type HalUartTx = uarte::UarteTx<'static>;
pub type HalUartRx = uarte::UarteRx<'static>;
pub type HalUartError = uarte::Error;

/// Two-Wire Interface Master (I2C) type.
///
/// Used for communication with the SI47xx radio chip.
pub type HalTwim = twim::Twim<'static>;

/// GPIO output pin type.
///
/// Used for control signals such as radio reset and LED indicators.
pub type HalOutput = Output<'static>;

pub fn hal_init() {
    let _ = embassy_nrf::init(Default::default());
}
