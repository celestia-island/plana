//! Tagged payload enums: transport endpoints and protocol-agnostic data
//! addresses.
//!
//! Field names and types are transcribed verbatim from the evernight
//! anchors (`evernight/src/protocol/backend.rs` at the Tier 3 wire-anchor
//! commit). Only the tagging differs by design: internal tagging on
//! `"kind"` with lowercased variant names, so TypeScript consumers get a
//! plain discriminated union (see the crate docs for the decision record).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Describes how to reach a device: the `transport` field of
/// [`protocol.connect`](crate::PROTOCOL_CONNECT_METHOD) and
/// [`protocol.probe`](crate::PROTOCOL_PROBE_METHOD) params.
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
/// `address` field of [`protocol.read`](crate::PROTOCOL_READ_METHOD) and
/// [`protocol.write`](crate::PROTOCOL_WRITE_METHOD) params.
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
/// [`protocol.write`](crate::PROTOCOL_WRITE_METHOD) result.
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
