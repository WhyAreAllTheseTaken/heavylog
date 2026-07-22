#![no_std]

//! A `log` implementation for no-std targets.
//! Use the `HeavyLogger` struct to initialise a logging system with a custom IO implementation.
//! ```ignore
//! use heavylog::LogOutput;
//! use log::info;
//! 
//! struct CustomOutput;
//! 
//! impl LogOutput for CustomOutput {
//!     type Error = ...;
//! 
//!     fn write_log_bytes(&self, data: &[u8]) -> Result<(), Self::Error> {
//!         // Insert IO implementation here.
//!         ...
//!     }
//! 
//!     fn flush(&self) -> Result<(), Self::Error> {
//!         ...
//!     }
//! }
//! 
//! fn main() {
//!     // Setup logger
//!     HeavyLogger::new(CustomOutput)
//!         .with_color(true)
//!         .init();
//! 
//!     info!("Log initialised successfully");
//! }
//! ```
//! 
//! ## Time Source Support
//! 
//! Support for times in logs can also be added by implementing the `TimeSource` trait.

use core::{error::Error, time::Duration};

use alloc::{boxed::Box, format, string::String};
use chrono::{DateTime, Utc};
use log::{Level, Log, SetLoggerError};

extern crate alloc;

/// A logging implementation.
pub struct HeavyLogger<O, T> {
    output: O,
    time_source: T,
    boot_time: DateTime<Utc>,
    relative_time: bool,
    color: bool
}

impl <O> HeavyLogger<O, NullTimeSource> {
    /// Create a new heavy logger with the specified output implementation.
    pub fn new(output: O) -> Self {
        Self {
            output,
            time_source: NullTimeSource,
            boot_time: NullTimeSource.current_time(),
            relative_time: true,
            color: true
        }
    }
}

impl <O, T> HeavyLogger<O, T> {
    /// Use the specified time source to get the current time.
    pub fn with_time_source<U: TimeSource>(self, time_source: U) -> HeavyLogger<O, U> {
        HeavyLogger {
            output: self.output,
            boot_time: time_source.current_time(),
            time_source: time_source,
            relative_time: self.relative_time,
            color: self.color
        }
    }

    /// If true, time is displayed as the number of seconds since boot.
    /// If false, time is displayed as the absolute time in Utc.
    pub fn with_relative_time(mut self, relative_time: bool) -> Self {
        self.relative_time = relative_time;

        self
    }
    
    /// If true, colour output is used using ANSI colour codes.
    /// If false, colour output is disable
    pub fn with_color(mut self, color: bool) -> Self {
        self.color = color;

        self
    }
} 

impl <O: LogOutput + Send + Sync + 'static, T: TimeSource + Send + Sync + 'static> HeavyLogger<O, T> {
    pub fn init(self) -> Result<(), SetLoggerError> {
        let logger = Box::leak(Box::new(self));

        log::set_logger(logger)
    }
}

impl <O: LogOutput + Send + Sync, T: TimeSource + Send + Sync> Log for HeavyLogger<O, T> {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let time = self.time_source.current_time();

        let mut output = String::new();

        if self.color {
            output += "\x1b[0m";
            output += match record.level() {
                Level::Error => "\x1b[0;91m",
                Level::Warn => "\x1b[0;93m",
                Level::Info => "\x1b[0;97m",
                Level::Debug => "\x1b[0;37m",
                Level::Trace => "\x1b[0;94m",
            };
        }

        output += &if self.relative_time {
            let duration = time.signed_duration_since(&self.boot_time);

            let duration = duration.to_std().unwrap_or(Duration::ZERO);

            let duration = format!("{duration:?}");

            format!("[{duration:>10}] ")
        } else {
            let time = format!("{time}");
            
            format!("[{time:>20}] ")
        };

        output += match record.level() {
            Level::Error => "[E] ",
            Level::Warn => "[W] ",
            Level::Info => "[i] ",
            Level::Debug => "[d] ",
            Level::Trace => "[t] ",
        };

        output += &format!("{}", record.args());

        if self.color {
            output += "\x1b[0m";
        }

        output += "\n";

        self.output.write_log_bytes(output.as_bytes()).expect("failed to write log entry");
    }

    fn flush(&self) {
        self.output.flush().expect("failed to flush log");
    }
}

/// Methods requires for outputting log data.
pub trait LogOutput {
    /// The error if a logging error occurs.
    type Error: Error;

    /// Write out the specified data to the logging system.
    fn write_log_bytes(&self, data: &[u8]) -> Result<(), Self::Error>;

    /// Flush the log output.
    fn flush(&self) -> Result<(), Self::Error>;
}

/// A source of the current time.
pub trait TimeSource {
    /// Get the current local time.
    fn current_time(&self) -> DateTime<Utc>;
}

/// A time source that just returns a default time.
pub struct NullTimeSource;

impl TimeSource for NullTimeSource {
    fn current_time(&self) -> DateTime<Utc> {
        DateTime::UNIX_EPOCH
    }
}
