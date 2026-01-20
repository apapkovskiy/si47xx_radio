use core::cell::RefCell;
use critical_section::Mutex;
use embedded_io::Write;

use embassy_nrf::uarte;

struct SerialPort<'a>(&'a Mutex<RefCell<Option<uarte::UarteTx<'a>>>>);

static WRITER_MUTEX: Mutex<RefCell<Option<uarte::UarteTx<'static>>>> = Mutex::new(RefCell::new(None));
static WRITER_OUT: SerialPort = SerialPort(&WRITER_MUTEX);

pub fn stdout_get() -> StdOut {
    StdOut
}

pub fn stdout_init(tx: uarte::UarteTx<'static>) {
    WRITER_OUT.init(tx);
}

impl<'a> SerialPort<'a> {
    fn init(&'a self, tx: uarte::UarteTx<'a>) {
        critical_section::with(|cs| {
            self.0.borrow_ref_mut(cs).replace(tx);
        });
    }
    fn write(&self, buf: &[u8]) -> Result<usize, uarte::Error> {
        critical_section::with(|cs| {
            // This code runs within a critical section.
            if let Some(tx) = self.0.borrow_ref_mut(cs).as_mut() {
                let _ =tx.blocking_write(buf);
            }
            Ok(buf.len())
       })
    }
}

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
