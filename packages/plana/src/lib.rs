//! PLANA - Protocol for Live Agent Network Automation: a typed
//! application-layer protocol for real-time state synchronization and control
//! between a client shell and a backend service runtime, built on JSON-RPC 2.0
//! (the way HTTP is built on TCP). Not a general-purpose RPC framework.
//!
//! `plana` is an umbrella crate that re-exports the standalone publishable
//! crates and adds one optional server-side module:
//!
//! - [`plana_protocol_core`](plana-protocol-core) — the generic protocol core
//!   (health/network descriptors, RBAC, region policy, identity, base
//!   messages, handshake primitives), re-exported at the crate root and
//!   always available. The JSON-RPC envelope lives in the framing crate
//!   (`plana-jsonrpc`), re-exported as the `plana::jsonrpc` module.
//! - `plana-celestia-types` — the celestia platform domain profile (agent,
//!   task, panel, industrial and tool domain messages), re-exported at the
//!   crate root behind the `celestia` feature (default on).
//! - `plana-jsonrpc` — the JSON-RPC 2.0 framing layer, re-exported as the
//!   `jsonrpc` module.
//! - `plana-evernight-client` — the evernight terminal-route dispatch client
//!   (P83/M1 routing contract), re-exported as the `evernight_client` module.
//! - `plana-protocol-core`'s `tracing_helpers` (behind the `tracing-helpers`
//!   feature) — `ShortTimer` formatting for `tracing-subscriber`.
//!
//! Module names shared between the core and the domain profile (`http`,
//! `enums`) are merged below so both the generic and the domain types stay
//! reachable under one path (`plana::http::HealthResponse` and
//! `plana::http::AgentItem`); `protocol` keeps the generic protocol module.
//! The full domain surface is always available at `plana::celestia`.
#[cfg(feature = "celestia")]
pub use plana_celestia_types::*;
#[cfg(feature = "tracing-helpers")]
pub use plana_protocol_core::tracing_helpers;
pub use plana_protocol_core::*;
#[cfg(feature = "celestia")]
pub mod celestia {
    pub use plana_celestia_types::*;
}

// The generic protocol module (base messages, handshake primitives). The
// domain profile's protocol module stays reachable at
// `plana::celestia::protocol`.
pub use plana_protocol_core::protocol;

// Merged modules — names shared by the core and the domain profile. The two
// globs below are disjoint by construction (core = generic types, domain =
// platform types); adding a type with the same name to both crates is a
// compile error here, which is the intended guard.
pub mod http {
    #[cfg(feature = "celestia")]
    pub use plana_celestia_types::http::*;
    pub use plana_protocol_core::http::*;
}
pub mod enums {
    #[cfg(feature = "celestia")]
    pub use plana_celestia_types::enums::*;
    pub use plana_protocol_core::enums::*;
}

pub mod jsonrpc {
    pub use plana_jsonrpc::*;
}

// The evernight terminal-route dispatch client (P83/M1 routing contract;
// consumed by entelecheia's PoleMos adapter). Follows the same re-export
// pattern as the `jsonrpc` module above.
pub mod evernight_client {
    pub use plana_evernight_client::*;
}

#[cfg(feature = "rpc-server")]
pub mod rpc_server;
