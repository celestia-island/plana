//! evernight Tier 3 `protocol.*` plugin-contract wire DTOs.
//!
//! The authoritative JSON shapes are pinned by
//! `tests/evernight_wire_shapes.rs`. TypeScript consumers get these from
//! `@celestia-island/plana-types` (`bindings/evernightProtocol.ts`), the
//! family's single protocol package. (The standalone
//! `@celestia-island/plana-evernight-protocol` package and its shim crate
//! `plana_evernight_protocol` were retired — evernight's gateway now imports
//! this module directly.)

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Describes how to reach a device: the `transport` field of
/// [`protocol.connect`](PROTOCOL_CONNECT_METHOD) and
/// [`protocol.probe`](PROTOCOL_PROBE_METHOD) params.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, export_to = "evernightProtocol.ts")]
pub enum TransportInfoDto {
    /// TCP endpoint (host, port).
    Tcp {
        /// TCP hostname or IP address.
        host: String,
        /// TCP port number.
        port: u16,
    },
    /// Serial port (device path, optional baud rate hint).
    Serial {
        /// Serial device path (e.g. "/dev/ttyUSB0").
        port: String,
        /// Optional baud rate hint. Serializes as `null` when absent and
        /// defaults to `None` when the field is missing on the wire.
        baud: Option<u32>,
    },
}

/// Identifies a data location across any industrial protocol: the
/// `address` field of [`protocol.read`](PROTOCOL_READ_METHOD) and
/// [`protocol.write`](PROTOCOL_WRITE_METHOD) params.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export, export_to = "evernightProtocol.ts")]
pub enum DataAddressDto {
    /// Modbus: station ID + function code + register address range.
    Modbus {
        /// Modbus station/slave ID.
        station: u8,
        /// Modbus function code.
        fc: u8,
        /// Starting register address.
        address: u16,
        /// Number of registers to read/write.
        count: u16,
    },
    /// S7comm: DB number + byte offset + length.
    S7 {
        /// Data block number.
        db_number: u16,
        /// Byte offset within the DB.
        offset: u16,
        /// Number of bytes to read/write.
        length: u16,
    },
    /// MC Protocol (Mitsubishi): device code + head address + count.
    Mc {
        /// Device code (e.g. "D", "M", "X", "Y").
        device: String,
        /// Starting head address.
        head_address: u32,
        /// Number of elements.
        count: u16,
    },
    /// Generic raw address (for future protocols).
    Raw {
        /// Raw address string (protocol-specific).
        address: String,
        /// Expected data size in bytes.
        size: usize,
    },
}

/// Tri-state confirmation of a
/// [`protocol.write`](PROTOCOL_WRITE_METHOD) result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "evernightProtocol.ts")]
pub enum WriteVerificationDto {
    /// A subsequent read-back verified the written value.
    Confirmed,
    /// A read-back was attempted but did not verify the write.
    Unconfirmed,
    /// The protocol has no read-back capability, so verification status is
    /// unknown (distinct from a silent `false`).
    #[default]
    Unknown,
}

// (dto.rs 与 methods.rs 合并为同一模块,类型直接可见,不再需要 crate::dto 路径。)

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
///
/// JSON-RPC 2.0 lets a no-argument method omit `params` entirely, and the
/// sibling TypeScript client does exactly that when called without an
/// argument; plana's router hands such a handler `Value::Null` for the
/// missing field. Deserialization therefore tolerates `null` alongside the
/// canonical `{}` (unknown fields are ignored, mirroring serde's derive
/// default), while serialization always emits `{}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, TS)]
#[ts(export, export_to = "evernightProtocol.ts")]
pub struct PingParamsDto {}

impl<'de> Deserialize<'de> for PingParamsDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PingParamsDto;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("the empty params object `{}` or null")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(PingParamsDto {})
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(PingParamsDto {})
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while map
                    .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
                    .is_some()
                {}
                Ok(PingParamsDto {})
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

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
