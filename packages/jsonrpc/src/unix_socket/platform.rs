use anyhow::{Context, Result};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub use interprocess::local_socket::{
    Listener as InterprocessListener, ListenerNonblockingMode, Stream as InterprocessStream,
    traits::Listener as InterprocessAccept,
};
use interprocess::local_socket::{
    prelude::*,
    {GenericFilePath, ListenerOptions, Stream as LocalSocketStream},
};
use tracing::{info, warn};

const SOCKET_PERMS: u32 = 0o777;

pub fn ensure_socket_dir(socket_path: &Path) -> Result<()> {
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create socket dir {}", parent.display()))?;

        chmod_recursive(parent, SOCKET_PERMS);
    }
    Ok(())
}

pub fn remove_stale_socket(socket_path: &Path) {
    if socket_path.exists()
        && let Err(e) = std::fs::remove_file(socket_path)
    {
        warn!(
            path = %socket_path.display(),
            error = %e,
            "failed to remove stale socket file"
        );
    }
}

pub fn chmod_socket(socket_path: &Path) {
    chmod_recursive(socket_path, SOCKET_PERMS);
}

#[cfg(unix)]
pub async fn bind_tokio(socket_path: &Path) -> Result<tokio::net::UnixListener> {
    ensure_socket_dir(socket_path)?;
    remove_stale_socket(socket_path);

    let listener = tokio::net::UnixListener::bind(socket_path)
        .with_context(|| format!("failed to bind Unix socket {}", socket_path.display()))?;

    chmod_socket(socket_path);

    info!(path = %socket_path.display(), "Unix socket bound (tokio)");
    Ok(listener)
}

#[cfg(unix)]
pub async fn connect_tokio(socket_path: &Path) -> Result<tokio::net::UnixStream> {
    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("failed to connect to Unix socket {}", socket_path.display()))?;
    info!(path = %socket_path.display(), "Connected to Unix socket (tokio)");
    Ok(stream)
}

pub fn bind_interprocess(socket_path: &Path) -> Result<interprocess::local_socket::Listener> {
    ensure_socket_dir(socket_path)?;
    remove_stale_socket(socket_path);

    let name = socket_path
        .to_fs_name::<GenericFilePath>()
        .with_context(|| {
            format!(
                "failed to convert path to socket name: {}",
                socket_path.display()
            )
        })?;

    let listener = ListenerOptions::new()
        .name(name)
        .create_sync()
        .with_context(|| {
            format!(
                "failed to create LocalSocket listener at {}",
                socket_path.display()
            )
        })?;

    chmod_socket(socket_path);

    info!(path = %socket_path.display(), "Unix socket bound (interprocess)");
    Ok(listener)
}

pub fn connect_interprocess(socket_path: &Path) -> Result<interprocess::local_socket::Stream> {
    let name = socket_path
        .to_fs_name::<GenericFilePath>()
        .with_context(|| {
            format!(
                "failed to convert path to socket name: {}",
                socket_path.display()
            )
        })?;

    let stream = LocalSocketStream::connect(name)
        .with_context(|| format!("failed to connect to {}", socket_path.display()))?;

    info!(path = %socket_path.display(), "Connected to Unix socket (interprocess)");
    Ok(stream)
}

#[cfg(unix)]
fn chmod_recursive(path: &Path, mode: u32) {
    if let Ok(metadata) = std::fs::metadata(path) {
        let mut perms = metadata.permissions();
        perms.set_mode(mode);
        if let Err(e) = std::fs::set_permissions(path, perms) {
            warn!(
                path = %path.display(),
                error = %e,
                mode = format!("{:#o}", mode),
                "failed to set permissions"
            );
        }
    }
}

#[cfg(not(unix))]
fn chmod_recursive(_path: &Path, _mode: u32) {}
