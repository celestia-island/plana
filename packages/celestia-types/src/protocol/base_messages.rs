//! Base protocol messages — heartbeat, error, ack.
//!
//! The base message set is generic protocol-core vocabulary; this module is a
//! re-export so `plana::celestia::protocol::base_messages` (and the domain
//! crate root) resolve to the same canonical types as the generic core. The
//! canonical definitions live in `plana_protocol_core::protocol::base_messages`
//! (including their TypeScript bindings export).

pub use plana_protocol_core::protocol::base_messages::*;
