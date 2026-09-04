use tracing::{Level, subscriber::set_global_default};
use tracing_subscriber::{
    FmtSubscriber,
    filter::LevelFilter,
    fmt::format::{Compact, DefaultFields, Format, Writer},
    fmt::time::{FormatTime, SystemTime},
    prelude::*,
    reload::{self, Handle},
};

pub struct ConfigurableTimer(bool);

impl FormatTime for ConfigurableTimer {
    fn format_time(&self, writer: &mut Writer<'_>) -> std::fmt::Result {
        if self.0 {
            SystemTime.format_time(writer)
        } else {
            Ok(())
        }
    }
}

type ReloadHandle =
    Handle<LevelFilter, FmtSubscriber<DefaultFields, Format<Compact, ConfigurableTimer>>>;

/// Init logger with provided level and configurable timestamps
pub fn init_logger(initial_level: Level, with_time: bool) -> ReloadHandle {
    let filter = LevelFilter::from_level(initial_level);
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    let builder = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .compact()
        .with_timer(ConfigurableTimer(with_time));

    let subscriber = builder.finish().with(filter_layer);

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
