//! The archive manifest: the metadata document that anchors integrity
//! (`payload_sha256`) and, when encrypted, doubles as the AEAD
//! additional authenticated data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The manifest schema generation. Must be `1`; any other value is
/// rejected on read (the forward-compat tripwire, mirroring the
/// akivili manifest convention).
pub const ARCHIVE_MANIFEST_SCHEMA: u32 = 1;

/// How the payload is protected at rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Encryption {
    /// Plain tar; integrity rides the manifest SHA-256 alone.
    None,
    /// AES-256-GCM; manifest JSON is the AEAD additional authenticated
    /// data, so a swapped manifest breaks decryption.
    Aes256Gcm,
}

impl Encryption {
    /// The manifest's wire spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Aes256Gcm => "aes-256-gcm",
        }
    }

    /// Parse the manifest's wire spelling.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "none" => Some(Self::None),
            "aes-256-gcm" => Some(Self::Aes256Gcm),
            _ => None,
        }
    }
}

/// The metadata document beside every archive payload (`manifest.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveManifest {
    /// Manifest schema generation; must be
    /// [`ARCHIVE_MANIFEST_SCHEMA`].
    pub schema: u32,
    /// The project-local namespace this archive belongs to (e.g.
    /// `plugins`, `configs`, `deploy`).
    pub kind: String,
    /// The archived subject's stable id (e.g. the plugin id).
    pub id: String,
    /// Creation instant (UTC, RFC 3339).
    pub created_at: DateTime<Utc>,
    /// Where the archive was taken from (a host/role label, never a
    /// credential).
    pub source: String,
    /// SHA-256 of the payload bytes as stored (post-encryption when
    /// encrypted — this is what sits on disk).
    pub payload_sha256: String,
    /// Payload size in bytes as stored.
    pub payload_bytes: u64,
    /// How the payload is protected.
    pub encryption: Encryption,
    /// Number of entries the tar carries (directories and files).
    pub entry_count: u64,
    /// Free-form, secret-free metadata (caller-supplied labels the
    /// restore review can show).
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub meta: std::collections::BTreeMap<String, String>,
}

impl ArchiveManifest {
    /// The canonical JSON text — both the on-disk manifest bytes and
    /// the AEAD additional authenticated data. One serialization, two
    /// uses, so they can never drift apart.
    pub fn canonical_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// The human-facing locator (`kind/id/stamp`) used in errors.
    pub fn locator(&self, stamp: &str) -> String {
        format!("{}/{}/{}", self.kind, self.id, stamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ArchiveManifest {
        ArchiveManifest {
            schema: ARCHIVE_MANIFEST_SCHEMA,
            kind: "plugins".into(),
            id: "acme-office".into(),
            created_at: Utc::now(),
            source: "uninstall".into(),
            payload_sha256: "ab".repeat(32),
            payload_bytes: 128,
            encryption: Encryption::Aes256Gcm,
            entry_count: 3,
            meta: std::collections::BTreeMap::from([("version".into(), "0.1.0".into())]),
        }
    }

    #[test]
    fn encryption_wire_round_trip() {
        for enc in [Encryption::None, Encryption::Aes256Gcm] {
            assert_eq!(Encryption::parse(enc.as_str()), Some(enc));
        }
        assert_eq!(Encryption::parse("plain"), None);
    }

    #[test]
    fn canonical_json_is_stable_and_covers_the_digest() {
        let m = sample();
        let a = m.canonical_json().unwrap();
        let b = m.canonical_json().unwrap();
        assert_eq!(a, b, "two serializations of one manifest must agree");
        assert!(
            a.contains(&m.payload_sha256),
            "the payload digest rides the canonical JSON (tamper-evident)"
        );
        let back: ArchiveManifest = serde_json::from_str(&a).unwrap();
        assert_eq!(back, m);
    }
}
