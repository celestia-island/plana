//! The Sync protocol - a typed application-layer protocol for real-time
//! state synchronization and control between a client shell and a backend
//! service runtime.
//!
//! The layering mirrors HTTP over TCP: JSON-RPC 2.0 supplies the generic
//! framing and transports (the "TCP" of the stack), while the Sync protocol
//! defines what messages *mean* - connection handshakes, hierarchical state
//! snapshots and patches, agent lifecycle events, human-in-the-loop review,
//! catalog synchronization, skill chains and device control (the "HTTP" of
//! the stack). If you only need generic remote calls, use a plain JSON-RPC
//! framework; `plana` implements a concrete application protocol.
//!
//! `plana` is an umbrella crate that re-exports the two standalone
//! publishable crates and adds one optional server-side module:
pub use plana_types::*;

pub mod jsonrpc {
    pub use plana_jsonrpc::*;
}

#[cfg(feature = "rpc-server")]
pub mod rpc_server;
