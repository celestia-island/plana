use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;

use tracing::{info, warn};

use plana::jsonrpc::unix_socket::log_socket_dir;
use plana_config::UserConfig;

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

/// Open `path` for writing, creating it with `mode` in the `open(2)` call
/// itself.
///
/// The token is a bearer credential: creating the file with the umask default
/// and `chmod`ing it right after leaves a window in which the secret is readable
/// by anyone the umask allowed, so the mode goes into the create call instead
/// (credential hygiene, AGENTS.md red line 1).
///
/// Windows keeps its ACL-based permissions; mode bits have no meaning there.
fn open_private_file(path: &PathBuf, mode: u32) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        options.mode(mode);
    }
    #[cfg(not(unix))]
    {
        let _ = mode; // suppress unused-variable warning on Windows
    }
    options.open(path)
}

fn write_token_with_perms(token: &str, path: &PathBuf, mode: u32) -> std::io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = match open_private_file(path, mode) {
        Ok(file) => file,
        Err(e) => {
            if path.exists() {
                match std::fs::remove_file(path) {
                    Ok(()) => open_private_file(path, mode)?,
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
    };
    // open(2) ignores `mode` for an existing file, so the descriptor is
    // fchmod'ed too: handle-scoped (no path can be swapped under it) and it
    // repairs a token file a previous release left at 0o644.
    #[cfg(unix)]
    file.set_permissions(std::fs::Permissions::from_mode(mode))?;
    file.write_all(token.as_bytes())?;
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode_of(path: &std::path::Path) -> u32 {
        std::fs::metadata(path)
            .expect("token file metadata")
            .permissions()
            .mode()
            & 0o777
    }

    /// The token is a bearer credential: it must be owner-only from the instant
    /// the file exists, both for a fresh file and for one an older release left
    /// at 0o644.
    #[test]
    fn write_token_with_perms_creates_and_tightens_owner_only() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("scepter.token");

        std::fs::create_dir_all(path.parent().expect("token parent")).expect("create token dir");

        // The create call itself must carry the mode: a follow-up chmod lands
        // too late by construction, so observe the file before any tightening.
        drop(open_private_file(&path, 0o600).expect("create token file"));
        assert_eq!(
            mode_of(&path),
            0o600,
            "a newly created connection token must be owner-only from the first byte"
        );
        write_token_with_perms("test-token", &path, 0o600).expect("write fresh token");
        assert_eq!(
            mode_of(&path),
            0o600,
            "rewriting the connection token must keep it owner-only"
        );

        // Pre-existing file from an older release: open(2) ignores mode for an
        // existing file, so the descriptor-level chmod must tighten it.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("widen token file");
        write_token_with_perms("rotated-token", &path, 0o600).expect("rewrite token");
        assert_eq!(
            mode_of(&path),
            0o600,
            "an existing 0o644 connection token must be tightened"
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("read token"),
            "rotated-token"
        );
    }
}
