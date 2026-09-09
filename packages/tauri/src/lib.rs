//! # plana-tauri — the Tauri adapter + standard command battery for the
//! PLANA relay extension.
//!
//! Any Tauri app adopts the relay profile (see
//! `plana-rpc-client::relay` and `docs/en/rpc/relay-profile.md`) by adding
//! this one dependency:
//!
//! - [`battery`] pre-mounts the standard `relay.*` system handlers —
//!   proxy, entrypoint, connections, forwarding edits, and host window
//!   controls — onto a [`RelayHost`](plana_rpc_client::relay::RelayHost);
//! - with the `bridge` feature (default), [`bridge`] exposes the two
//!   fixed transport functions as Tauri commands/events: `bridge_call`
//!   and the `bridge_event` lane, wired to that host.
//!
//! Apps keep full freedom: the battery mounts nothing you don't ask for,
//! extra local namespaces mount through the host's own API, and the
//! forwarding whitelist starts empty (deny-by-default).

pub mod battery;

#[cfg(feature = "bridge")]
pub mod bridge;

pub use battery::{BatteryHooks, mount_battery};
