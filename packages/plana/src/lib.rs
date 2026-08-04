//! PLANA - Protocol for Live Agent Network Automation: a typed
//! application-layer protocol for real-time state synchronization and control
//! between a client shell and a backend service runtime, built on JSON-RPC 2.0
//! (the way HTTP is built on TCP). Not a general-purpose RPC framework.
//!
//! `plana` is an umbrella crate that re-exports the two standalone
//! publishable crates and adds one optional server-side module:
#[cfg(feature = "celestia")]
pub use plana_celestia_types::*;
#[cfg(feature = "celestia")]
pub mod celestia {
    pub use plana_celestia_types::*;
}

pub mod jsonrpc {
    pub use plana_jsonrpc::*;
}

#[cfg(feature = "rpc-server")]
pub mod rpc_server;
