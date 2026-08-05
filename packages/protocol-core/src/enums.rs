//! Generic connection-topology vocabulary.
//!
//! Enumerations that are part of the protocol vocabulary itself, independent
//! of any specific platform domain. Platform-specific vocabulary lives in the
//! domain profile crates (e.g. `plana-celestia-types`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

/// How a peer reached this instance — the topology of the link, not the
/// physical medium (which is platform-specific, e.g. `LinkType` in an
/// industrial gateway).
///
/// `Local` covers both a native peer and a same-host virtualized peer; the
/// two are distinguished by a shared-secret handshake, not by IP alone.
/// `RemoteLan` is an RFC1918 / link-local peer without that secret;
/// `RemoteInternet` is anything else.
///
/// This enum is the canonical source of truth. Platform profiles attach it
/// to their sessions as a routing tag — any authorized session is stamped
/// with how it connected — but the protocol itself does not branch on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "enums.ts")]
pub enum ConnectionType {
    Local,
    RemoteLan,
    RemoteInternet,
}

impl ConnectionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionType::Local => "local",
            ConnectionType::RemoteLan => "remote_lan",
            ConnectionType::RemoteInternet => "remote_internet",
        }
    }
}

impl fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<ConnectionType> for String {
    fn from(v: ConnectionType) -> String {
        v.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_type_as_str_values() {
        assert_eq!(ConnectionType::Local.as_str(), "local");
        assert_eq!(ConnectionType::RemoteLan.as_str(), "remote_lan");
        assert_eq!(ConnectionType::RemoteInternet.as_str(), "remote_internet");
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", ConnectionType::Local), "local");
        assert_eq!(
            format!("{}", ConnectionType::RemoteInternet),
            "remote_internet"
        );
    }

    #[test]
    fn from_enum_to_string_matches_as_str() {
        let s: String = ConnectionType::RemoteLan.into();
        assert_eq!(s, "remote_lan");
    }

    #[test]
    fn serde_round_trip_each_variant() {
        for ct in [
            ConnectionType::Local,
            ConnectionType::RemoteLan,
            ConnectionType::RemoteInternet,
        ] {
            let s = serde_json::to_string(&ct).unwrap();
            let back: ConnectionType = serde_json::from_str(&s).unwrap();
            assert_eq!(back, ct);
        }
    }

    #[test]
    fn serde_serializes_variant_name_not_as_str() {
        let s = serde_json::to_string(&ConnectionType::RemoteLan).unwrap();
        assert_eq!(s, r#""RemoteLan""#);
        assert_ne!(s, r#""remote_lan""#);
    }

    #[test]
    fn serde_rejects_unknown_variant() {
        assert!(serde_json::from_str::<ConnectionType>(r#""remote""#).is_err());
    }
}
