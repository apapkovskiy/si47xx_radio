//! Board support module for SI47xx radio.
//!
//! This module provides hardware abstraction layer (HAL) implementations for different
//! development boards. It enables the SI47xx radio application to run on various
//! hardware platforms with minimal code changes.
//!
//! # Supported Boards
//!
//! - **nRF5340 DK**: Nordic Semiconductor nRF5340 Development Kit (feature: `nrf5340dk`)
//!
//! # Architecture
//!
//! The module uses conditional compilation to include board-specific implementations:
//! - Common nRF HAL types and interrupts are defined in `nrfcommon`
//! - Board-specific initialization functions are in board-specific modules
//!
//! # Usage
//!
//! The appropriate board module is automatically selected at compile time based on
//! the enabled features.

#[cfg(feature = "nrf")]
pub mod hal {
    //! Hardware abstraction layer for nRF-based boards.
    //!
    //! This module contains common HAL types and board-specific implementations
    //! for nRF microcontroller family.

    pub mod nrfcommon;
    pub use nrfcommon::*;
    pub mod nrf5340dk;
    pub use nrf5340dk::*;
}

#[cfg(feature = "host")]
pub mod hal {
    //! Hardware abstraction layer for host builds.
    pub mod hosthal;
    pub use embedded_hal::digital::OutputPin;
    pub use hosthal::*;
    pub fn hal_init() {
        // No hardware initialization needed for host builds
    }
}
