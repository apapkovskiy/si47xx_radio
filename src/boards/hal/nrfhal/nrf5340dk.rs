#![cfg(feature = "nrf5340dk")]
//! HAL implementation for nRF5340 Development Kit.
//!
//! This module provides hardware initialization functions specific to the Nordic
//! Semiconductor nRF5340 Development Kit. It configures all the peripherals needed
//! for the SI47xx radio application:
//!
//! - **Radio Reset**: GPIO P1.03 (active high)
//! - **LED**: GPIO P0.28 (active low)
//! - **I2C/TWI**: SERIAL1 peripheral with SCL on P1.13 and SDA on P1.14
//! - **UART**: SERIAL0 peripheral at 115200 baud with RXD on P0.22 and TXD on P0.20
//! - **QSPI Flash**: QSPI peripheral with 32MHz frequency and 4-line I/O mode
//!
//! # Pin Configuration
//!
//! The pin assignments follow the nRF5340 DK standard pinout and are compatible
//! with the SI47xx radio shield or breakout board connections.
//!
//! # Safety
//!
//! All peripheral initialization functions use `unsafe` operations to steal peripherals.
//! This is safe as long as each peripheral is only initialized once and used exclusively
//! by the returned instance.

use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::peripherals;
use embassy_nrf::{qspi, twim, uarte};
use static_cell::ConstStaticCell;

use crate::boards::hal::*;

/// Creates the radio reset output pin.
///
/// Configures GPIO P1.03 as an output with initial high level (radio active).
/// The SI47xx radio requires a reset pulse (low-high transition) to initialize.
///
/// # Returns
///
/// A configured GPIO output that controls the radio's reset line.
pub fn hal_radio_reset_create() -> HalOutput {
    let reset_pin = unsafe { peripherals::P1_03::steal() };
    Output::new(reset_pin, Level::High, OutputDrive::Standard)
}

/// Creates the LED output pin.
///
/// Configures GPIO P0.28 as an output with initial low level (LED off).
/// This LED is typically used as a status indicator for the application.
///
/// # Returns
///
/// A configured GPIO output that controls an onboard LED.
pub fn hal_led_create() -> HalOutput {
    let led = unsafe { peripherals::P0_28::steal() };
    Output::new(led, Level::Low, OutputDrive::Standard)
}

/// Creates the I2C/TWI interface for radio communication.
///
/// Configures SERIAL1 peripheral as TWI Master with:
/// - SCL: P1.13
/// - SDA: P1.14
/// - Default configuration (100 kHz)
/// - 16-byte RAM buffer for EasyDMA
///
/// This interface is used to communicate with the SI47xx radio chip via I2C protocol.
///
/// # Returns
///
/// A configured TWI master instance ready for I2C communication.
pub fn hal_twi_create() -> HalTwim {
    let config = twim::Config::default();
    let twi = unsafe { peripherals::SERIAL1::steal() };
    let scl = unsafe { peripherals::P1_13::steal() };
    let sda = unsafe { peripherals::P1_14::steal() };
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    twim::Twim::new(twi, Irqs, sda, scl, config, RAM_BUFFER.take())
}

/// Creates the UART interface for console communication.
///
/// Configures SERIAL0 peripheral as UART with:
/// - Baud rate: 115200
/// - Parity: None
/// - RXD: P0.22
/// - TXD: P0.20
///
/// This interface is used for console output, debugging, and user interaction.
///
/// # Returns
///
/// A configured UART instance ready for serial communication.
pub fn hal_uart_create() -> (HalUartTx, HalUartRx) {
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;
    let serial = unsafe { peripherals::SERIAL0::steal() };
    let rxd = unsafe { peripherals::P0_22::steal() };
    let txd = unsafe { peripherals::P0_20::steal() };
    let ppi_ch0 = unsafe { peripherals::PPI_CH0::steal() };
    let ppi_ch1 = unsafe { peripherals::PPI_CH1::steal() };
    let timer = unsafe { peripherals::TIMER0::steal() };
    uarte::Uarte::new(serial, rxd, txd, Irqs, config).split_with_idle(timer, ppi_ch0, ppi_ch1)
}

/// Creates the QSPI flash interface for persistent storage.
///
/// Configures the QSPI peripheral with:
/// - Read opcode: FASTREAD
/// - Write opcode: PP
/// - Page size: 256 bytes
/// - Pins:
///   - SCK: P0.17 (Clock)
///   - CSN: P0.18 (Chip Select)
///   - IO0-IO3: P0.13-P0.16 (Data lines)
///
/// This interface provides access to the onboard QSPI flash memory for storing
/// radio settings, presets, and configuration data.
///
/// # Returns
///
/// A configured QSPI instance ready for flash memory operations.
pub fn hal_qspi_create() -> HalQspi {
    let mut config = qspi::Config::default();
    config.read_opcode = qspi::ReadOpcode::FASTREAD;
    config.write_opcode = qspi::WriteOpcode::PP;
    config.write_page_size = qspi::WritePageSize::_256BYTES;
    config.frequency = qspi::Frequency::M8;
    config.capacity = 8 * 1024 * 1024; // 8 MB
    let qspi = unsafe { peripherals::QSPI::steal() };
    let sck = unsafe { peripherals::P0_17::steal() };
    let csn = unsafe { peripherals::P0_18::steal() };
    let io0 = unsafe { peripherals::P0_13::steal() };
    let io1 = unsafe { peripherals::P0_14::steal() };
    let io2 = unsafe { peripherals::P0_15::steal() };
    let io3 = unsafe { peripherals::P0_16::steal() };
    qspi::Qspi::new(qspi, Irqs, sck, csn, io0, io1, io2, io3, config)
}
