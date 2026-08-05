//! Platform-specific JSON-RPC error codes (Rust-only; no TypeScript export).
//!
//! NOTE: The canonical wire-layer copy of the shared error codes lives in
//! `plana-jsonrpc` (`plana_jsonrpc::types::error_codes`); the standard
//! JSON-RPC 2.0 codes are also duplicated in the generic envelope copy in
//! `plana-protocol-core` (`plana_protocol_core::protocol::jsonrpc`); neither
//! copy exports TypeScript bindings.
//! the two generic copies are kept in sync manually. Unlike the base protocol
//! messages (`protocol::base_messages`), which this crate re-exports from
//! `plana-protocol-core` so there is a single canonical source, these codes
//! stay defined here because the domain error space (-32000 range) is
//! platform-specific. The celestia platform's custom codes are also
//! duplicated in `plana_jsonrpc::types::error_codes` (the canonical wire
//! copy) and kept in sync manually.

pub mod error_codes {
    pub const SNAPSHOT_FAILED: i64 = -32001;
    pub const AGENT_UNAVAILABLE: i64 = -32002;
    pub const CONTAINER_ERROR: i64 = -32003;
    pub const REPL_ERROR: i64 = -32004;
    pub const AUTH_ERROR: i64 = -32005;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_error_codes_unchanged() {
        assert_eq!(error_codes::SNAPSHOT_FAILED, -32001);
        assert_eq!(error_codes::AGENT_UNAVAILABLE, -32002);
        assert_eq!(error_codes::CONTAINER_ERROR, -32003);
        assert_eq!(error_codes::REPL_ERROR, -32004);
        assert_eq!(error_codes::AUTH_ERROR, -32005);
    }
}
