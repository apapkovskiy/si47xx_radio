//! Lightweight `log` backend that writes colorized messages over the serial console.
//!
//! The logger:
//! - emits millisecond timestamps from `embassy_time::Instant`,
//! - colorizes levels with the escape sequences from `console::console_colors`,
//! - caps verbosity at `Level::Info` (debug/trace are ignored),
//! - writes through the shared UART writer provided by `console::stdout_get()`.

use crate::console;
use crate::console::console_colors::{RED, RESET, WHITE, YELLOW};
use core::fmt::Write as _;
use embassy_time::Instant;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SerialLogger;

impl SerialLogger {
    pub const fn new() -> Self {
        Self
    }
    /// Map a `Level` to its ANSI color escape for pretty printing.
    pub const fn get_level_color(level: Level) -> core::fmt::Arguments<'static> {
        match level {
            Level::Error => RED,
            Level::Warn => YELLOW,
            Level::Info => WHITE,
            Level::Debug => WHITE,
            Level::Trace => WHITE,
        }
    }
}

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let seconds = Instant::now().as_millis();
            let level_color = SerialLogger::get_level_color(record.level());
            let _ = write!(
                console::stdout_get(),
                "{level_color}[{:012}] <{}> {}: {}{RESET}\r\n",
                seconds,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.args()
            );
        }
    }
    fn flush(&self) {}
}

static LOGGER: SerialLogger = SerialLogger::new();

/// Install the serial logger and set the max level to `Info`.
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
