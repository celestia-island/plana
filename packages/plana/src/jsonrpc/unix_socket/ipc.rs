use anyhow::Result;
#[cfg(unix)]
use std::path::Path;

use super::platform;

pub fn bind_log_socket() -> Result<interprocess::local_socket::Listener> {
    let path = super::log_socket::log_socket_path();
    platform::bind_interprocess(&path)
}

pub fn connect_log_socket() -> Result<interprocess::local_socket::Stream> {
    let path = super::log_socket::log_socket_path();
    platform::connect_interprocess(&path)
}

#[cfg(unix)]
pub async fn bind_jsonrpc_socket(socket_path: &Path) -> Result<tokio::net::UnixListener> {
    platform::bind_tokio(socket_path).await
}

#[cfg(unix)]
pub async fn connect_jsonrpc_socket(socket_path: &Path) -> Result<tokio::net::UnixStream> {
    platform::connect_tokio(socket_path).await
}
