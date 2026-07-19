use std::path::PathBuf;

pub const DEFAULT_SOCKET_DIR: &str = "/run/entelecheia";
pub const LOG_SOCKET_FILENAME: &str = "entelecheia-log.sock";

pub fn log_socket_dir() -> PathBuf {
    std::env::var("ENTELECHEIA_SOCKET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_SOCKET_DIR))
}

pub fn log_socket_path() -> PathBuf {
    log_socket_dir().join(LOG_SOCKET_FILENAME)
}
