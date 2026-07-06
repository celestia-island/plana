//! Cross-platform machine fingerprint.
//!
//! Provides a deterministic, installation-unique identifier that can be read
//! without admin privileges. The fingerprint is used as key material for the
//! local OAuth issuer — the same machine always derives the same key, so the
//! issuer and the resource server (both running on the same host) can
//! independently compute the same HS256 signing/verification key.
//!
//! The raw platform identifier is hashed with SHA-256 so the output is a
//! fixed-width, opaque hex string regardless of the underlying source format.
//!
//! # Sources
//!
//! | Platform | Source |
//! |----------|--------|
//! | Linux    | `/etc/machine-id` (systemd) |
//! | Windows  | `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` |
//! | macOS    | `IOPlatformUUID` via ioreg |
//! | WSL2     | `/etc/machine-id` (WSL2 distro's own) |
//!
//! If the primary source is unavailable, falls back to a composite of the
//! hostname + OS details — less unique but stable across reboots.

use sha2::{Digest, Sha256};

/// 64 hex chars — the SHA-256 hash of the machine identifier.
pub fn machine_fingerprint() -> Option<String> {
    let raw = raw_machine_id()?;
    let hash = Sha256::digest(raw.as_bytes());
    Some(hex::encode(hash))
}

/// Return the best available raw platform identifier, or None.
fn raw_machine_id() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // Standard systemd machine-id; present on virtually all modern Linux
        // distros AND inside WSL2 distros (each WSL2 distro gets its own).
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
        // Fallback: /var/lib/dbus/machine-id (older systems).
        if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // MachineGuid is a REG_SZ under HKLM; it's set at Windows
        // installation and never changes. Reading it does not require admin.
        if let Ok(output) = std::process::Command::new("reg")
            .args([
                "query",
                r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            // Output format:
            //   HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography
            //       MachineGuid    REG_SZ    {xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}
            for line in text.lines() {
                if let Some(pos) = line.find("REG_SZ") {
                    let val = line[pos + 6..].trim();
                    // Strip curly braces if present.
                    let val = val.trim_start_matches('{').trim_end_matches('}');
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-d2", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if let Some(pos) = line.find("IOPlatformUUID") {
                    let remainder = &line[pos..];
                    // Line looks like: "IOPlatformUUID" = "XXXXXXXX-XXXX-..."
                    if let Some(q1) = remainder.find('"') {
                        let after_key = &remainder[q1 + 1..];
                        if let Some(q2) = after_key.find('"') {
                            let after_eq = &after_key[q2 + 1..];
                            if let Some(q3) = after_eq.find('"') {
                                if let Some(q4) = after_eq[q3 + 1..].find('"') {
                                    let uuid = &after_eq[q3 + 1..q3 + 1 + q4];
                                    return Some(uuid.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Universal fallback: hostname + OS. Not unique across clones, but stable
    // across reboots and sufficient for a dev machine where the primary source
    // failed for some reason.
    let host = hostname::get().ok().and_then(|h| h.into_string().ok())?;
    Some(format!(
        "{}:{}:{}",
        host,
        std::env::consts::OS,
        std::env::consts::ARCH
    ))
}

/// Thin wrapper — hostname detection without pulling in a crate.
/// Windows: COMPUTERNAME env (always set). Unix: /proc/sys/kernel/hostname
/// or /etc/hostname. Fallback: "unknown".
mod hostname {
    use std::ffi::OsString;

    pub fn get() -> std::io::Result<OsString> {
        #[cfg(windows)]
        {
            std::env::var("COMPUTERNAME")
                .map(OsString::from)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "COMPUTERNAME"))
        }
        #[cfg(unix)]
        {
            for path in ["/proc/sys/kernel/hostname", "/etc/hostname"] {
                if let Ok(s) = std::fs::read_to_string(path) {
                    let trimmed = s.trim().to_string();
                    if !trimmed.is_empty() {
                        return Ok(OsString::from(trimmed));
                    }
                }
            }
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "hostname files"))
        }
        #[cfg(not(any(windows, unix)))]
        {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "unsupported platform"))
        }
    }
}
