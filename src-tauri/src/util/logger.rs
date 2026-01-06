use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

/// Initialize the logger with the given verbosity
pub fn init(verbose: bool) {
    let mut builder = Builder::from_default_env();

    if verbose {
        builder.filter_level(LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }

    // Set up custom format
    builder.format(|buf, record| {
        writeln!(
            buf,
            "[{}] {} - {}",
            record.level(),
            buf.timestamp(),
            record.args()
        )
    });

    builder.init();
}

/// Log a message with the "log!" macro
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}
