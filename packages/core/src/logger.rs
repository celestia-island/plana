// Log initialization

use std::path::PathBuf;

use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize logger
pub fn init_logger() {
    let env_filter = EnvFilter::from_default_env().add_directive(Level::INFO.into());

    fmt().with_env_filter(env_filter).json().init();
}

/// Initialize logger (text format)
pub fn init_logger_text() {
    let env_filter = EnvFilter::from_default_env().add_directive(Level::INFO.into());

    fmt().with_env_filter(env_filter).pretty().init();
}

/// Initialize logger (TUI mode, writes to temp file)
pub fn init_logger_tui() -> Result<String, std::io::Error> {
    let log_dir = std::env::var("LOG_DIR")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string());

    let log_dir = PathBuf::from(log_dir);
    std::fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join("entelecheia-tui.log");

    let env_filter = EnvFilter::from_default_env().add_directive(Level::INFO.into());

    let file_appender = tracing_appender::rolling::never(log_dir, "entelecheia-tui.log");

    fmt()
        .with_env_filter(env_filter)
        .with_writer(file_appender)
        .json()
        .init();

    Ok(log_file.to_string_lossy().to_string())
}
