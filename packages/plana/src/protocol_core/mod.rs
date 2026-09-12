//! Generic protocol core of PLANA (absorbed from the former
//! `plana-protocol-core` crate; the standalone name remains as a re-export
//! shim) — the shared wire types that any platform profile builds on.
//!
//! This module owns the platform-independent message set of the PLANA
//! protocol:
//!
//! - the base protocol messages ([`protocol::base_messages`]),
//! - connection handshake / version / identity negotiation primitives
//!   ([`protocol::handshake`]),
//! - health and network descriptors ([`http`]),
//! - RBAC permission/role types ([`rbac`]),
//! - regional compliance policy ([`region`]),
//! - cross-platform identity (machine fingerprint, [`identity`]),
//! - generic connection-topology vocabulary ([`enums`]).
//!
//! The generic JSON-RPC 2.0 envelope intentionally does NOT live here: its
//! single canonical definition is the [`crate::jsonrpc`] module of this same
//! crate. A former copy here was removed after the two drifted apart.
//!
//! Types carry serde (Serialize and/or Deserialize) and, where applicable,
//! JSON-Schema (`schemars::JsonSchema`) and TypeScript-bindings (`ts-rs`)
//! support; e.g. the http descriptors are Serialize + TS only, while the
//! protocol message types are full serde round-trip types.
//!
//! This crate knows nothing about any specific platform domain. Domain
//! profiles (e.g. `plana-celestia-types`) depend on the `plana` foundation
//! and plug their per-domain message vocabularies in; `plana` re-exports this
//! core at its crate root and as `plana::protocol_core`, and does not
//! re-export any domain profile.

// ── Module tree ─────────────────────────────────────────────
// The generic wire vocabulary lives under a small set of folders:
//   protocol/ — base messages, handshake (WS transport)
//   http/     — health/network/status descriptors and generic REST DTOs
// and a few single-file modules at the root (enums, identity, rbac, region).
// The glob re-exports at the bottom keep every type reachable at the module
// root (`plana::protocol_core::TypeName` — and, through the re-export shim,
// `plana_protocol_core::TypeName`).
pub mod enums;
pub mod http;
pub mod identity;
pub mod protocol;
pub mod rbac;
pub mod region;

#[cfg(feature = "tracing-helpers")]
pub mod tracing_helpers;

pub use http::{BackendKind, HealthResponse, NetworkInfo, ServiceStatus};

// protocol/ — transport core
pub use protocol::base_messages::*;
pub use protocol::handshake::*;

// enums/ — generic vocabulary enums (ConnectionType, …)
pub use enums::*;

// region/ — regional compliance policy types
pub use region::*;

/// Protocol version advertised by the platform.
pub const PROTOCOL_VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_is_one_point_zero() {
        assert_eq!(PROTOCOL_VERSION, "1.0.0");
    }

    #[test]
    fn core_flat_reexports_resolve() {
        // Guard the umbrella-facing surface: generic health types, base
        // messages, handshake primitives, region policy and connection
        // topology must all be reachable at the crate root.
        let _ = BackendKind::Prod;
        let _ = HealthResponse::ok("1.0.0", BackendKind::Dev, 1, NetworkInfo::unknown());
        let _ = ServiceStatus::Ok;
        let _ = BaseHeartbeatParams { timestamp: 0 };
        let _ = HandshakeAckParams {
            ok: true,
            error: None,
        };
        let _ = PingParams { timestamp: 0 };
        assert_eq!(HANDSHAKE_VERSION, 1);
        let _ = RegionPolicy::FreeMarket;
        assert_eq!(ConnectionType::Local.as_str(), "local");
    }
}
