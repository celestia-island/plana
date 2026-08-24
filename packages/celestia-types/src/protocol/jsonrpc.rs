//! Platform-specific JSON-RPC error codes.
//!
//! The single canonical definition of these codes lives in `plana-jsonrpc`
//! (`plana_jsonrpc::types::error_codes`, the wire crate); this module simply
//! re-exports the platform-specific (-32000 range) subset so the domain
//! profile never duplicates them. The generic JSON-RPC 2.0 envelope and the
//! standard error codes also live in `plana-jsonrpc` — a former second copy
//! in `plana-protocol-core` was removed after the copies drifted apart.

pub mod error_codes {
    pub use plana_jsonrpc::types::error_codes::{
        AGENT_UNAVAILABLE, AUTH_ERROR, CONTAINER_ERROR, REPL_ERROR, SNAPSHOT_FAILED,
    };
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
