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

// ═══════════════════════════════════════════════════════════════
// Gateway manifest (evernight tier-3 reverse proxy, GW-1b)
// ═══════════════════════════════════════════════════════════════
//
// The manifest is the single source of truth for what a tier-3 evernight
// node serves publicly (design: `_reports/evernight-gateway-design-2026-09-18.md` §5).
// It is a TOML document (hand-writable by operators, generated by chest),
// stored per exit-node in the registry's persistence, and pushed down the
// relay channel via `Config.Apply`. Types live here (not evernight-local)
// because chest (Rust, typed writer) and evernight-gw (typed reader) both
// consume them — the "≥2 services" upstream precondition is met.

/// Top-level manifest: identifies the document, pins the executing node,
/// and carries the server blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct GatewayManifest {
    /// Schema version (for forward-compatible evolution).
    pub version: u32,
    /// Unique manifest id (for idempotent push/pull and audit).
    pub id: String,
    /// Who produced this manifest ("chest-admin" or "hand-written").
    #[serde(default)]
    pub generated_by: String,
    /// The tier-3 node that must execute this manifest (I2 invariant).
    pub exit_node: String,
    /// Server blocks: one per public hostname.
    #[serde(default)]
    pub servers: Vec<GatewayServer>,
}

/// One public hostname's routing tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct GatewayServer {
    /// Public hostname(s) this server block answers (SNI + Host match).
    pub host: Vec<String>,
    /// Listen address (default `0.0.0.0:443`).
    #[serde(default = "default_listen")]
    pub listen: String,
    /// TLS termination config.
    #[serde(default)]
    pub tls: Option<GatewayTls>,
    /// Redirect plain HTTP to HTTPS (adds a :80 listener).
    #[serde(default = "default_true")]
    pub redirect_http: bool,
    /// Ordered routes (first match wins).
    #[serde(default)]
    pub routes: Vec<GatewayRoute>,
}

fn default_listen() -> String {
    "0.0.0.0:443".to_string()
}

/// TLS material reference: the cert/key live on the tier-3 node's local
/// secure storage; the manifest only carries a **reference** (the door's
/// host), never the private key bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct GatewayTls {
    /// `static` (file-based, managed by acme.sh + install-cert) — ACME
    /// in-process is a future variant.
    #[serde(rename = "type")]
    pub mode: String,
    /// Reference to the door's certificate (e.g. "door:s3.celestia.world").
    pub cert: String,
    /// Reference to the door's private key (same node-local storage).
    pub key: String,
}

/// One route: a structured matcher + a discriminated action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct GatewayRoute {
    /// Route id (for logs and mutation targeting).
    #[serde(default)]
    pub id: String,
    /// Structured match table (fields are AND'd; arrays within a field are OR'd).
    #[serde(default)]
    pub r#match: GatewayMatch,
    /// What to do when the route matches (discriminated by `type`).
    pub action: GatewayAction,
    /// Named middleware to apply (references or inline).
    #[serde(default)]
    pub use_: Vec<String>,
    /// Rate limit for this route.
    #[serde(default)]
    pub ratelimit: Option<GatewayRateLimit>,
    /// Request body size ceiling.
    #[serde(default)]
    pub body_max: Option<String>,
    /// Request timeout.
    #[serde(default)]
    pub timeout: Option<String>,
}

/// Structured route matcher. Fields AND across, arrays OR within.
/// Rejects string-DSL patterns (traefik-style) — everything is
/// structurally validated at parse time.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct GatewayMatch {
    /// Path prefixes to match (any of).
    #[serde(default)]
    pub path_prefix: Vec<String>,
    /// Exact paths (any of).
    #[serde(default)]
    pub path: Vec<String>,
    /// HTTP methods (any of).
    #[serde(default)]
    pub methods: Vec<String>,
    /// Header presence/value match.
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

/// Route action, discriminated by the `type` field (serde internally-tagged).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "evernightGateway.ts")]
pub enum GatewayAction {
    /// Reverse-proxy to upstream(s).
    Proxy {
        /// Upstream URL(s); multiple = load-balanced (P1 feature).
        upstream: Vec<String>,
        /// Load-balancing strategy (default round_robin).
        #[serde(default = "default_lb")]
        lb: String,
        /// **Critical for SigV4**: preserve the original Host header.
        /// rustfs presigned URLs verify the Host; changing it = 403.
        #[serde(default = "default_true")]
        preserve_host: bool,
    },
    /// Return a fixed response.
    Static {
        status: u16,
        #[serde(default)]
        body: String,
    },
    /// Redirect to another URL.
    Redirect {
        to: String,
        #[serde(default = "default_redirect_status")]
        status: u16,
    },
    /// Reject with a status code (e.g. 403 for deny).
    Reject { status: u16 },
}

fn default_lb() -> String {
    "round_robin".to_string()
}
fn default_true() -> bool {
    true
}
fn default_redirect_status() -> u16 {
    308
}

/// Per-route rate limit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct GatewayRateLimit {
    pub requests: u32,
    pub per: String,
    #[serde(default = "default_rate_key")]
    pub key: String,
}

fn default_rate_key() -> String {
    "ip".to_string()
}

/// The `Config.Apply` downlink payload: the manifest + its version for
/// idempotent push (replayed old versions are refused by the node).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct ConfigApplyParams {
    pub manifest: GatewayManifest,
    /// Monotonically increasing version (refuse <= applied).
    pub config_version: u64,
}

/// Node-side acknowledgment pushed back up the relay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "evernightGateway.ts")]
pub struct ConfigApplyAck {
    pub applied_version: u64,
    pub manifest_id: String,
}
