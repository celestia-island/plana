//! Wire DTOs for the evernight Tier 3 `protocol.*` plugin contract.
//!
//! evernight's Tier 3 plugin lane lets third-party protocol backends run as
//! external processes speaking JSON-RPC 2.0 over WebSocket or a Unix domain
//! socket. The generic JSON-RPC framing has lived upstream in
//! the `plana` crate's `jsonrpc` module for a while; what was missing — and
//! what this crate provides — are the five method payloads themselves. They
//! are defined here once so that every side of the contract shares one
//! source of truth.
//!
//! # Consumers
//!
//! - **The evernight gateway** (Rust): the future `RemotePluginBackend` /
//!   akivili plugin-registry wave speaks these DTOs to plugin processes.
//!   The gateway's own in-tree backend types (`TransportInfo`, `DataAddress`,
//!   `ProtocolWriteResult` in `evernight::protocol::backend`) are the
//!   runtime-side anchors these DTOs were transcribed from.
//! - **Tier 3 plugin authors, in any language**: the JSON shapes pinned by
//!   this crate's tests *are* the contract. A Python, Go, C or Node.js
//!   plugin implements a JSON-RPC server answering the five methods below;
//!   TypeScript authors can consume the generated bindings
//!   (`bindings/evernightProtocol.ts`, emitted by `cargo test` via ts-rs)
//!   instead of hand-writing the types.
//!
//! # The five methods
//!
//! | Method constant | Method | Params → Result |
//! |---|---|---|
//! | [`PROTOCOL_CONNECT_METHOD`] | `protocol.connect` | [`ConnectParamsDto`] → [`ConnectResultDto`] |
//! | [`PROTOCOL_READ_METHOD`] | `protocol.read` | [`ReadParamsDto`] → [`ReadResultDto`] |
//! | [`PROTOCOL_WRITE_METHOD`] | `protocol.write` | [`WriteParamsDto`] → [`WriteResultDto`] |
//! | [`PROTOCOL_PING_METHOD`] | `protocol.ping` | [`PingParamsDto`] → [`PingResultDto`] |
//! | [`PROTOCOL_PROBE_METHOD`] | `protocol.probe` | [`ProbeParamsDto`] → [`ProbeResultDto`] |
//!
//! # Tagging decision
//!
//! The payload enums ([`TransportInfoDto`], [`DataAddressDto`]) use serde
//! **internal tagging** (`"kind"`, lowercased variant names) so TypeScript
//! consumers get a natural discriminated union
//! (`{ kind: "tcp", host, port } | { kind: "serial", port, baud }`). This
//! deliberately diverges from serde's default external tagging, which
//! evernight's in-tree anchors snapshot today; those anchors and the Tier 3
//! guide's interface table migrate to these shared DTOs together. Field
//! names and types are transcribed verbatim from the evernight anchors, so
//! only the tag spelling changes on the wire.
//!
//! # Name-collision note (polemos)
//!
//! `plana-celestia-types` ships an unrelated
//! `plana_celestia_types::tools::polemos::ProtocolProbeResult` — the LLM
//! terminal tool's probe report (always carries `host`/`port`/`banner`/
//! `details`, `confidence: f64`). This crate's [`ProbeResultDto`] is the
//! `protocol.probe` *wire* result (`protocol` + `confidence: f32`). They
//! live in different crates and namespaces by design; do not merge or
//! re-export one as the other.

pub mod dto;
pub mod methods;

pub use dto::{DataAddressDto, TransportInfoDto, WriteVerificationDto};
pub use methods::{
    ConnectParamsDto, ConnectResultDto, PROTOCOL_CONNECT_METHOD, PROTOCOL_PING_METHOD,
    PROTOCOL_PROBE_METHOD, PROTOCOL_READ_METHOD, PROTOCOL_WRITE_METHOD, PingParamsDto,
    PingResultDto, ProbeParamsDto, ProbeResultDto, ReadParamsDto, ReadResultDto, WriteParamsDto,
    WriteResultDto,
};
