# Heavy Log

A `log` implementation for no-std targets.
Use the `HeavyLogger` struct to initialise a logging system with a custom IO implementation.
```rust
use heavylog::LogOutput;
use log::info;

struct CustomOutput;

impl LogOutput for CustomOutput {
    type Error = ...;

    fn write_log_bytes(&self, data: &[u8]) -> Result<(), Self::Error> {
        // Insert IO implementation here.
        ...
    }

    fn flush(&self) -> Result<(), Self::Error> {
        ...
    }
}

fn main() {
    // Setup logger
    HeavyLogger::new(CustomOutput)
        .with_color(true)
        .init();

    info!("Log initialised successfully");
}
```

## Time Source Support

Support for times in logs can also be added by implementing the `TimeSource` trait.

