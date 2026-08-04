//! Scaffolding for a typed bidirectional sync protocol.
//!
//! `plana` is an umbrella crate that re-exports the two standalone
//! publishable crates and adds one optional server-side module:
//!
//! - **`plana-types`** — shared wire types: the JSON-RPC envelope
//!   ([`protocol`]), transport-agnostic message params (`ws/`), per-tool MCP
//!   I/O structs (`mcp/`), health/network descriptors ([`http`]), identity,
//!   region policy, RBAC, and model metadata. Every type is
//!   `Serialize`/`Deserialize` with JSON Schema and TypeScript bindings
//!   generation support.
//! - **`plana-jsonrpc`** — the JSON-RPC 2.0 wire machinery: request/response/
//!   notification types, a method registry ([`jsonrpc::rpc_router::RpcMethodMap`]),
//!   Unix-domain-socket transports ([`jsonrpc::unix_socket`],
//!   [`jsonrpc::unix_transport`]), and a typed bridge between method strings
//!   and serde-serializable messages ([`jsonrpc::bridge`]).
//! - **`rpc-server`** (feature `rpc-server`) — server-side session management
//!   ([`rpc_server::SessionManager`]), SSE event streaming, and request
//!   network/geolocation detection ([`rpc_server::detect_network`]).
//!
//! Everything from `plana-types` is re-exported at the crate root, so
//! `plana::http::...`, `plana::protocol::...`, `plana::ws::...` and all
//! root-level type names keep resolving exactly as before. `plana-jsonrpc`
//! is re-exported as the `jsonrpc` module, preserving `plana::jsonrpc::...`
//! paths. Consumers that only need the wire types or the JSON-RPC machinery
//! can depend on `plana-types` / `plana-jsonrpc` directly.
//!
//! This is not a general-purpose RPC framework: it is the scaffold for one
//! specific typed bidirectional sync protocol.
//!
//! The `plana-types` side follows a strict rule: a type belongs there only
//! when it is defined as the canonical source of truth and consumed on both
//! sides of a wire protocol. Anything else stays out.

pub use plana_types::*;

pub mod jsonrpc {
    pub use plana_jsonrpc::*;
}

#[cfg(feature = "rpc-server")]
pub mod rpc_server;
