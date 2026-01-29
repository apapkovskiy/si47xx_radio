//! Minimal console output wrapper around the nRF UARTE peripheral.
//!
//! The module provides a global `StdOut` handle that implements both
//! `embedded_io::Write` and `core::fmt::Write`, so the rest of the
//! firmware can use `write!`/`writeln!` macros or raw byte writes without
//! touching the HAL types directly. Output is protected by a
//! `critical_section::Mutex` to keep logging cheap and safe in interrupt
//! contexts.

use core::cell::RefCell;
use critical_section::Mutex;
use embedded_io::Write;

use embassy_nrf::uarte;

/// Thin wrapper that stores a shared UARTE TX handle and exposes a
/// `Write`-compatible API.
struct SerialPort<'a>(&'a Mutex<RefCell<Option<uarte::UarteTx<'a>>>>);

static WRITER_MUTEX: Mutex<RefCell<Option<uarte::UarteTx<'static>>>> =
    Mutex::new(RefCell::new(None));
static WRITER_OUT: SerialPort = SerialPort(&WRITER_MUTEX);

pub mod console_colors {
    #![allow(dead_code)]
    //! ANSI color escape codes for optional terminal styling.
    use core::fmt::Arguments;
    pub const RESET: Arguments = format_args!("\x1B[0m");
    pub const EMPTY: Arguments = format_args!("");
    pub const BLACK: Arguments = format_args!("\x1B[0;30m");
    pub const RED: Arguments = format_args!("\x1B[0;31m");
    pub const GREEN: Arguments = format_args!("\x1B[0;32m");
    pub const YELLOW: Arguments = format_args!("\x1B[0;33m");
    pub const BLUE: Arguments = format_args!("\x1B[0;34m");
    pub const MAGENTA: Arguments = format_args!("\x1B[0;35m");
    pub const CYAN: Arguments = format_args!("\x1B[0;36m");
    pub const WHITE: Arguments = format_args!("\x1B[0;37m");
    pub const BOLD: Arguments = format_args!("\x1B[1m");
    pub const BOLD_BLACK: Arguments = format_args!("\x1B[1;30m");
    pub const BOLD_RED: Arguments = format_args!("\x1B[1;31m");
    pub const BOLD_GREEN: Arguments = format_args!("\x1B[1;32m");
    pub const BOLD_YELLOW: Arguments = format_args!("\x1B[1;33m");
    pub const BOLD_BLUE: Arguments = format_args!("\x1B[1;34m");
    pub const BOLD_MAGENTA: Arguments = format_args!("\x1B[1;35m");
    pub const BOLD_CYAN: Arguments = format_args!("\x1B[1;36m");
    pub const BOLD_WHITE: Arguments = format_args!("\x1B[1;37m");
}

pub fn stdout_get() -> StdOut {
    StdOut
}

/// Install the TX half of a configured UARTE instance as the global writer.
///
/// Call this once during startup after the peripheral has been initialized.
pub fn stdout_init(tx: uarte::UarteTx<'static>) {
    WRITER_OUT.init(tx);
}

impl<'a> SerialPort<'a> {
    /// Store the provided TX handle and emit a leading newline so that early
    /// logs start on a clean line. Safe to call only once during boot.
    fn init(&'a self, tx: uarte::UarteTx<'a>) {
        critical_section::with(|cs| {
            self.0.borrow_ref_mut(cs).replace(tx);
            self.write(b"\n").ok();
        });
    }
    /// Write a buffer to the UART if it has been initialized.
    ///
    /// The function always returns `Ok(buf.len())`; if UART TX is not yet
    /// installed the bytes are silently dropped. This keeps logging sites
    /// lightweight and failure-tolerant during early boot.
    fn write(&self, buf: &[u8]) -> Result<usize, uarte::Error> {
        critical_section::with(|cs| {
            // This code runs within a critical section.
            if let Some(tx) = self.0.borrow_ref_mut(cs).as_mut() {
                let _ = tx.blocking_write(buf);
            }
            Ok(buf.len())
        })
    }
}

/// Handle returned by `stdout_get` that implements both `embedded_io::Write`
/// and `core::fmt::Write` to simplify logging across the project.
pub struct StdOut;

impl embedded_io::ErrorType for StdOut {
    type Error = uarte::Error;
}

impl embedded_io::Write for StdOut {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        WRITER_OUT.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl core::fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        let _ = self.write(s.as_bytes());
        Ok(())
    }
}
