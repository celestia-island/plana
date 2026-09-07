//! WebSocket close codes and heartbeat method names of the service profile.

/// Normal closure.
pub const NORMAL: u16 = 1000;
/// A binary frame arrived on a text-only profile.
pub const UNSUPPORTED_DATA: u16 = 1003;
/// Connection-level policy violation (e.g. auth revoked mid-connection).
pub const POLICY_VIOLATION: u16 = 1008;
/// Unexpected server-side failure while servicing the connection.
pub const INTERNAL_ERROR: u16 = 1011;
/// No inbound frame (including heartbeats) within the idle window.
pub const IDLE_TIMEOUT: u16 = 4000;

/// Heartbeat notification the client sends (no `id`).
pub const HEARTBEAT_METHOD: &str = "Base.Heartbeat";
/// Heartbeat ack notification the server answers with on the control lane.
pub const HEARTBEAT_ACK_METHOD: &str = "Base.HeartbeatAck";
