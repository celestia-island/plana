use std::path::PathBuf;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use tracing::{info, warn};

use _config::UserConfig;
use _infra_jsonrpc::unix_socket::log_socket_dir;

const TOKEN_FILE_NAME: &str = "scepter.token";

pub fn connection_token_path() -> PathBuf {
    UserConfig::discover_config_root().join(TOKEN_FILE_NAME)
}

pub fn socket_token_path() -> PathBuf {
    log_socket_dir().join(TOKEN_FILE_NAME)
}

pub fn generate_token() -> String {
    use base64::Engine;
    use rand::Rng;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn write_token_to_file(token: &str) -> std::io::Result<()> {
    let config_path = connection_token_path();
    write_token_with_perms(token, &config_path, 0o600)?;

    let socket_path = socket_token_path();
    match write_token_with_perms(token, &socket_path, 0o600) {
        Ok(()) => {
            info!(
                event = "connection_token_written",
                config_path = %config_path.display(),
                socket_path = %socket_path.display(),
                "Connection token written to config dir and socket dir"
            );
        }
        Err(e) => {
            warn!(
                event = "connection_token_socket_write_failed",
                path = %socket_path.display(),
                error = %e,
                "Failed to write connection token to socket dir (non-fatal)"
            );
        }
    }

    Ok(())
}

fn write_token_with_perms(token: &str, path: &PathBuf, mode: u32) -> std::io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }
    if let Err(e) = std::fs::write(path, token) {
        if path.exists() {
            match std::fs::remove_file(path) {
                Ok(()) => std::fs::write(path, token)?,
                Err(rm_err) => {
                    warn!(
                        event = "connection_token_remove_failed",
                        path = %path.display(),
                        write_error = %e,
                        remove_error = %rm_err,
                        "Cannot write or remove old token file"
                    );
                    return Err(e);
                }
            }
        } else {
            return Err(e);
        }
    }
    // Unix: chmod the token file to the requested mode (typically 0o600).
    // Windows: file permissions are ACL-based, not mode bits — skip the chmod;
    // the token file is in a user-owned config dir and isn't world-readable.
    #[cfg(unix)]
    {
        let file_perms = std::fs::metadata(path)?.permissions();
        let mut perms = file_perms;
        perms.set_mode(mode);
        std::fs::set_permissions(path, perms)?;
    }
    #[cfg(not(unix))]
    {
        let _ = mode; // suppress unused-variable warning on Windows
    }
    Ok(())
}

pub fn read_token() -> Option<String> {
    let socket_path = socket_token_path();
    if socket_path.exists() {
        match std::fs::read_to_string(&socket_path) {
            Ok(token) => {
                let trimmed = token.trim().to_string();
                if !trimmed.is_empty() {
                    info!(
                        event = "connection_token_read",
                        path = %socket_path.display(),
                        "Read connection token from socket dir"
                    );
                    return Some(trimmed);
                }
            }
            Err(e) => {
                warn!(
                    event = "connection_token_read_failed",
                    path = %socket_path.display(),
                    error = %e,
                    "Failed to read connection token from socket dir"
                );
            }
        }
    }

    let config_path = connection_token_path();
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(token) => {
                let trimmed = token.trim().to_string();
                if !trimmed.is_empty() {
                    info!(
                        event = "connection_token_read",
                        path = %config_path.display(),
                        "Read connection token from config dir"
                    );
                    return Some(trimmed);
                }
            }
            Err(e) => {
                warn!(
                    event = "connection_token_read_failed",
                    path = %config_path.display(),
                    error = %e,
                    "Failed to read connection token from config dir"
                );
            }
        }
    }

    None
}

pub fn remove_token_file() {
    let config_path = connection_token_path();
    if config_path.exists()
        && let Err(e) = std::fs::remove_file(&config_path)
    {
        warn!(path = %config_path.display(), error = %e, "failed to remove connection config token");
    }
    let socket_path = socket_token_path();
    if socket_path.exists()
        && let Err(e) = std::fs::remove_file(&socket_path)
    {
        warn!(path = %socket_path.display(), error = %e, "failed to remove socket token");
    }
}
