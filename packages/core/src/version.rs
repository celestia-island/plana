//! Version management module
//!
//! Provides version checking to ensure TUI and server versions match.

/// Base version number (from Cargo.toml)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check version compatibility
pub fn is_compatible(server_version: &str, client_version: &str) -> bool {
    server_version == client_version
}

/// Version check result
#[derive(Debug, Clone)]
pub struct VersionCheckResult {
    pub compatible: bool,
    pub server_version: String,
    pub client_version: String,
}

/// Check version compatibility and return detailed info
pub fn check_version_compatibility(server_version: &str) -> VersionCheckResult {
    let client_version = VERSION;
    VersionCheckResult {
        compatible: is_compatible(server_version, client_version),
        server_version: server_version.to_string(),
        client_version: client_version.to_string(),
    }
}
