//! Simple logger that just writes to serial

use log::{LevelFilter, Log, SetLoggerError};

use crate::serial_println;

/// Logger
pub struct Logger {
    /// Level filter
    level: LevelFilter,
}

impl Logger {
    /// Constructs logger with given filter level
    pub const fn new(level: LevelFilter) -> Self {
        Self { level }
    }

    /// Initialises logger
    pub fn init(&'static self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.level);
        log::set_logger(self)
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level().to_level_filter() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            serial_println!(
                "{:>5} | {:>40} | {}",
                record.level(),
                record.module_path().unwrap(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
