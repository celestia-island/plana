//! The five `protocol.*` methods and their parameter/result DTOs.
//!
//! The method name constants give both sides of the contract one spelling;
//! the param/result structs mirror the Tier 3 guide's interface table:
//!
//! | Method | Params | Result |
//! |---|---|---|
//! | `protocol.connect` | `{ transport }` | `{ connected }` |
//! | `protocol.read` | `{ address }` | `{ raw, latency_us }` |
//! | `protocol.write` | `{ address, data }` | `{ confirmed, verification }` |
//! | `protocol.ping` | `{}` | `{ reachable }` |
//! | `protocol.probe` | `{ transport }` | `{ protocol, confidence }` |

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dto::{DataAddressDto, TransportInfoDto, WriteVerificationDto};

/// `protocol.connect` — open a connection to a device.
pub const PROTOCOL_CONNECT_METHOD: &str = "protocol.connect";
/// `protocol.read` — read raw bytes at a data address.
pub const PROTOCOL_READ_METHOD: &str = "protocol.read";
/// `protocol.write` — write raw bytes at a data address.
pub const PROTOCOL_WRITE_METHOD: &str = "protocol.write";
/// `protocol.ping` — quick connectivity check (lighter than a read).
pub const PROTOCOL_PING_METHOD: &str = "protocol.ping";
/// `protocol.probe` — auto-detect which protocol a transport speaks.
pub const PROTOCOL_PROBE_METHOD: &str = "protocol.probe";

/// Params of [`PROTOCOL_CONNECT_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct ConnectParamsDto {
    /// How to reach the device.
    pub transport: TransportInfoDto,
}

/// Result of [`PROTOCOL_CONNECT_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct ConnectResultDto {
    /// Whether the connection was established.
    pub connected: bool,
}

/// Params of [`PROTOCOL_READ_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct ReadParamsDto {
    /// Where to read.
    pub address: DataAddressDto,
}

/// Result of [`PROTOCOL_READ_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct ReadResultDto {
    /// Raw bytes read from the device.
    pub raw: Vec<u8>,
    /// Read latency in microseconds.
    pub latency_us: u64,
}

/// Params of [`PROTOCOL_WRITE_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct WriteParamsDto {
    /// Where to write.
    pub address: DataAddressDto,
    /// Raw bytes to write.
    pub data: Vec<u8>,
}

/// Result of [`PROTOCOL_WRITE_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct WriteResultDto {
    /// Whether a read-back verification confirmed the write. `false` is not
    /// proof of failure — see [`verification`](Self::verification) for the
    /// precise tri-state.
    pub confirmed: bool,
    /// Precise verification status. `#[serde(default)]` keeps legacy
    /// `{ "confirmed": bool }` payloads deserializable (they map to
    /// [`WriteVerificationDto::Unknown`]).
    #[serde(default)]
    pub verification: WriteVerificationDto,
}

/// Params of [`PROTOCOL_PING_METHOD`] — the empty params object `{}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct PingParamsDto {}

/// Result of [`PROTOCOL_PING_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct PingResultDto {
    /// Whether the device answered the connectivity check.
    pub reachable: bool,
}

/// Params of [`PROTOCOL_PROBE_METHOD`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct ProbeParamsDto {
    /// The endpoint to probe.
    pub transport: TransportInfoDto,
}

/// Result of [`PROTOCOL_PROBE_METHOD`].
///
/// Not to be confused with `plana_celestia_types::tools::polemos::
/// ProtocolProbeResult` (the LLM terminal tool's probe report) — see the
/// crate docs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct ProbeResultDto {
    /// Human-readable protocol name that was detected (e.g. "modbus_tcp").
    pub protocol: String,
    /// Detection confidence in `[0.0, 1.0]`.
    pub confidence: f32,
}
