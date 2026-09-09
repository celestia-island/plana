//! Relay extension to the service profile — the framework-agnostic core.
//!
//! See `docs/en/rpc/relay-profile.md`. This module standardizes the
//! single-hop contract so chains of any length compose: namespace
//! routing with a whitelist, the reserved `relay.*` system-control
//! namespace, the hop guard, layered proxy resolution, and the edge
//! channel contract (transport-agnostic — Tauri, egui, stdio adapters
//! all implement the same trait).

pub mod edge;
pub mod hops;
pub mod host;
pub mod names;
pub mod proxy;
pub mod router;

pub use edge::{EdgeListener, EdgeTransport, MemoryTransport};
pub use hops::HopGuard;
pub use host::{BridgeError, BridgeOutcome, LocalHandler, RelayHost, RpcSink};
pub use names::{namespace_of, relay_methods, Namespace, RELAY};
pub use proxy::{ProxyConfig, ProxyDecision};
pub use router::{ForwardingTable, Route};
