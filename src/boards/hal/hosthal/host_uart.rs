//! Host UART implementation for development and testing.
//!
//! This module provides a UART-like interface using standard input/output
//! for host-based testing. It's designed to simulate UART communication
//! on development machines without requiring actual hardware.

use async_std::io::Stdin;
use std::os::fd::AsRawFd;
extern crate std;
use super::{Error, HalUartRx, HalUartTx};
use std::io::Stdout as StdoutSync;

/// Implementation of embedded_io_async::Error for the host environment.
impl embedded_io_async::Error for Error {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match *self {
            Error::FakeError => embedded_io_async::ErrorKind::Other,
        }
    }
}

/// A blocking UART transmitter that writes to standard output.
///
/// This struct provides a synchronous interface for transmitting data
/// by writing to stdout. It's useful for host-based testing where UART
/// output is simulated using standard output.
pub struct HostUartTxBlocking(StdoutSync);
impl HostUartTxBlocking {
    pub fn new() -> Self {
        HostUartTxBlocking(std::io::stdout())
    }

    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        use std::io::Write;
        self.0.write(buf).map_err(|_| Error::FakeError)?;
        self.0.flush().map_err(|_| Error::FakeError)?;
        Ok(buf.len())
    }
}

impl Default for HostUartTxBlocking {
    fn default() -> Self {
        Self::new()
    }
}

/// A UART receiver that reads from standard input.
///
/// This struct provides an asynchronous interface for receiving data
/// by reading from stdin. It configures the terminal for raw mode to
/// allow for non-blocking reads without waiting for a newline. This
/// simulates UART reception in a host environment.
pub struct HostUartRx(Stdin);
impl HostUartRx {
    pub fn new() -> Self {
        let rx = async_std::io::stdin();
        // Configure terminal for raw mode (non-canonical, no echo).
        let fd = rx.as_raw_fd();
        let mut termios: libc::termios = unsafe { core::mem::zeroed() };
        unsafe { libc::tcgetattr(fd, &mut termios) };
        termios.c_lflag &= !(libc::ICANON | libc::ECHO);
        unsafe { libc::tcsetattr(fd, 0, &termios) };
        HostUartRx(rx)
    }
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        use async_std::io::prelude::ReadExt;
        self.0.read(buf).await.map_err(|_| Error::FakeError)
    }
}

impl Default for HostUartRx {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hal_uart_create() -> (HalUartTx, HalUartRx) {
    (HostUartTxBlocking::new(), HostUartRx::new())
}
