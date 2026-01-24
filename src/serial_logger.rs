use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};
use core::fmt::Write as _;
use embassy_time::Instant;
use crate::console;
use crate::console::console_colors::{RESET, RED, WHITE, YELLOW};

struct SerialLogger;

impl SerialLogger {
    pub const fn new() -> Self {
        Self
    }
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
            let _ = write!(console::stdout_get(), "{level_color}[{:012}] <{}> {}: {}{RESET}\r\n", seconds, record.level(), record.file().unwrap_or("unknown"), record.args());
        }
    }
    fn flush(&self) {}
}

static LOGGER: SerialLogger = SerialLogger::new();

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}
