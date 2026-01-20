use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};
use core::fmt::Write as _;
use crate::console;

struct SerialLogger;

impl SerialLogger {
    pub const fn new() -> Self {
        Self
    }
}

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let _ = write!(console::stdout_get(), "{}\r\n", record.args());
        }
    }
    fn flush(&self) {}
}

static LOGGER: SerialLogger = SerialLogger::new();

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}
