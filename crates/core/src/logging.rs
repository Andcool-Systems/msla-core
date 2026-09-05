use tracing::{Level, subscriber::set_global_default};
use tracing_subscriber::{
    Registry,
    filter::LevelFilter,
    fmt::time::SystemTime,
    prelude::*,
    reload::{self, Handle},
};

type ReloadHandle = Handle<LevelFilter, Registry>;

/// Init logger with provided level and configurable timestamps
pub fn init_logger(initial_level: Level, with_time: bool) -> ReloadHandle {
    let filter = LevelFilter::from_level(initial_level);
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    let fmt_layer = if with_time {
        tracing_subscriber::fmt::layer()
            .compact()
            .with_timer(SystemTime)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .compact()
            .without_time()
            .boxed()
    };

    let subscriber = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);

    set_global_default(subscriber).expect("Failed to set global default subscriber");

    reload_handle
}

/// Set logging level for existing logger
pub fn set_log_level(handle: ReloadHandle, level: tracing::Level) {
    handle
        .modify(|filter| *filter = LevelFilter::from_level(level))
        .expect("Failed to change log level");
}

/// Get log level from string
pub fn str_to_log_level(level: &str) -> Option<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Some(Level::TRACE),
        "debug" => Some(Level::DEBUG),
        "info" => Some(Level::INFO),
        "warn" => Some(Level::WARN),
        "error" => Some(Level::ERROR),
        _ => None,
    }
}
