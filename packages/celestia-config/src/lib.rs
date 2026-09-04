//! Canonical schema for the family-shared `~/.celestia/celestia.toml`.
//!
//! Every Celestia family tool (evernight-appliance flasher, chest desktop,
//! future helpers) keeps its own `[section]` in this one per-user file. The
//! schema here is the single source of truth for the modeled sections and the
//! file-level conventions:
//!
//! - **One section per tool.** A tool must only interpret its own section and
//!   treat every other section as opaque. `CelestiaConfig` models this with a
//!   typed [`CelestiaConfig::flasher`] plus a catch-all [`CelestiaConfig::other`]
//!   map, so a load → save round trip preserves sibling sections it does not
//!   understand.
//! - **Values are TOML-native.** Unlike earlier line-based writers, values keep
//!   their TOML types (strings stay quoted, tables stay tables).
//! - **Missing file is not an error.** Use [`load_or_default`] for the
//!   first-run experience, or [`load`] when a strict read is wanted.
//!
//! New sections belong here as typed structs only once at least two tools
//! consume them; until then they live in `CelestiaConfig::other` untouched.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Root of `~/.celestia/celestia.toml`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CelestiaConfig {
    /// `[flasher]` — evernight-appliance enrollment flasher.
    pub flasher: FlasherConfig,
    /// Every unmodeled section (and stray top-level key), preserved verbatim
    /// across load → save so family tools never clobber each other.
    #[serde(flatten)]
    pub other: BTreeMap<String, toml::Value>,
}

impl CelestiaConfig {
    /// Parse from raw TOML text.
    pub fn parse(text: &str) -> Result<Self, LoadError> {
        Ok(toml::from_str(text)?)
    }

    /// Serialize back to TOML text (pretty table ordering).
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::ser::to_string_pretty(self)
    }

    /// Read the file at `path`, or return an empty config when it does not
    /// exist or cannot be understood — the lenient contract the flasher has
    /// always used.
    pub fn load_or_default(path: &std::path::Path) -> Self {
        Self::load(path).unwrap_or_default()
    }

    /// Strict read: missing or malformed files are errors.
    pub fn load(path: &std::path::Path) -> Result<Self, LoadError> {
        let text = std::fs::read_to_string(path)?;
        Self::parse(&text)
    }

    /// Write the config, creating parent directories as needed. Unknown
    /// sibling sections held in [`CelestiaConfig::other`] are written back.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_toml().map_err(std::io::Error::other)?)
    }
}

/// `[flasher]` — the evernight-appliance enrollment flasher section.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FlasherConfig {
    /// UI locale, an IETF-style tag matching the hikari locale catalog
    /// (for example `"zh-Hans"`, `"en"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Keys the schema does not model yet, preserved verbatim on round trip
    /// so the flasher never loses a setting it stored before this crate
    /// learned about it.
    #[serde(flatten)]
    pub other: BTreeMap<String, toml::Value>,
}

/// Platform default path: `%USERPROFILE%\.celestia\celestia.toml` on Windows
/// (checked first, matching the flasher), `$HOME/.celestia/celestia.toml`
/// elsewhere, `./.celestia/celestia.toml` when neither variable is set.
#[must_use]
pub fn default_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    path_under(&home)
}

/// [`default_path`] rooted at an explicit home directory (testable variant).
#[must_use]
pub fn path_under(home: &str) -> PathBuf {
    PathBuf::from(home).join(".celestia").join("celestia.toml")
}

/// Failures from [`CelestiaConfig::load`].
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// The file could not be read.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The file exists but is not valid TOML (or not valid for this schema).
    #[error(transparent)]
    Parse(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flasher_locale() {
        let cfg = CelestiaConfig::parse("[flasher]\nlocale = \"zh-Hans\"\n").unwrap();
        assert_eq!(cfg.flasher.locale.as_deref(), Some("zh-Hans"));
        assert!(cfg.other.is_empty());
    }

    #[test]
    fn round_trip_preserves_unknown_sections() {
        let raw = "[flasher]\nlocale = \"en\"\n\n[mirror]\nendpoint = \"https://mirror.example\"\n";
        let cfg = CelestiaConfig::parse(raw).unwrap();
        assert_eq!(cfg.flasher.locale.as_deref(), Some("en"));
        let out = cfg.to_toml().unwrap();
        assert!(out.contains("[mirror]"), "sibling section dropped: {out}");
        assert!(out.contains("https://mirror.example"));
        let reparsed = CelestiaConfig::parse(&out).unwrap();
        assert_eq!(reparsed, cfg);
    }

    #[test]
    fn missing_file_yields_default() {
        let dir = std::env::temp_dir().join(format!("pcc-missing-{}", std::process::id()));
        let cfg = CelestiaConfig::load_or_default(&dir.join("absent.toml"));
        assert_eq!(cfg, CelestiaConfig::default());
    }

    #[test]
    fn malformed_file_lenient_vs_strict() {
        let dir = std::env::temp_dir().join(format!("pcc-bad-{}", std::process::id()));
        let path = dir.join("celestia.toml");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "[flasher\nbroken").unwrap();
        assert_eq!(
            CelestiaConfig::load_or_default(&path),
            CelestiaConfig::default()
        );
        assert!(CelestiaConfig::load(&path).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_creates_parents_and_reload_matches() {
        let dir = std::env::temp_dir().join(format!("pcc-save-{}", std::process::id()));
        let path = path_under(&dir.to_string_lossy());
        let mut cfg = CelestiaConfig::default();
        cfg.flasher.locale = Some("ja".to_string());
        cfg.save(&path).unwrap();
        assert_eq!(CelestiaConfig::load(&path).unwrap(), cfg);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flasher_section_preserves_unmodeled_keys() {
        let raw = "[flasher]\nlocale = \"en\"\nlast_device = \"sdA\"\n";
        let cfg = CelestiaConfig::parse(raw).unwrap();
        assert_eq!(
            cfg.flasher.other.get("last_device"),
            Some(&toml::Value::String("sdA".to_string()))
        );
        let out = cfg.to_toml().unwrap();
        assert!(out.contains("last_device"), "unmodeled key dropped: {out}");
    }

    #[test]
    fn path_under_matches_flasher_layout() {
        let p = path_under("/home/lab");
        assert_eq!(p, PathBuf::from("/home/lab/.celestia/celestia.toml"));
    }
}
