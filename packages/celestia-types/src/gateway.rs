//! Gateway service-profile wire types.
//!
//! Request/response payloads for the two gateway services that speak the
//! PLANA service profile (`docs/rpc/service-profile.md`):
//!
//! - **rescue** — gateway.celestia.world identity federation + emergency
//!   rescue channel (strict WS JSON-RPC; sessions are connection-bound,
//!   so no token is ever serialized to the client).
//! - **enrollment** — the evernight enrollment gateway (device pairing,
//!   parent signing certificates, device bootstrap). Field shapes are
//!   parity with today's HTTP structs in `evernight/packages/gateway`
//!   and the flasher's `enrollments.rs`, so the RPC migration is
//!   mechanical.
//!
//! Method mapping (service-profile naming, lowercase dotted):
//!
//! | Type | Method |
//! |---|---|
//! | [`RescueOpenSessionParams`] / [`RescueSessionOpened`] | `rescue.open_session` |
//! | [`QuotaState`] | `rescue.quota` (result) |
//! | [`RescueDiagnoseParams`] / [`RescueDiagnoseResult`] | `rescue.diagnose` |
//! | [`RescueDiagnoseProgressParams`] | `rescue.diagnose.progress` (notification) |
//! | [`GatewayServiceInfo`] | `gateway.info` |
//! | [`EnrollmentMintParams`] / [`EnrollmentMinted`] or [`EnrollmentMintParentChained`] | `enrollment.mint` |
//! | [`SigningCertParams`] / [`ParentCertIssued`] | `enrollment.signing_cert.mint` |
//! | [`EnrollmentStatus`] | `enrollment.status` (result) |
//! | [`DeviceBootstrap`] | `device.bootstrap` |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ═══════════════════════════════════════════════════════════════
// Rescue channel (gateway.celestia.world)
// ═══════════════════════════════════════════════════════════════

/// `rescue.open_session` params. The ticket is the one-shot opaque
/// credential issued by the OAuth callback redirect; it is consumed
/// exactly once at session mint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct RescueOpenSessionParams {
    pub ticket: String,
}

/// `rescue.open_session` result. The session handle itself is
/// connection-bound server state and never crosses the wire; the caller
/// learns only how long the bound session lives and the quota outlook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct RescueSessionOpened {
    /// Seconds until the bound session expires.
    pub expires_in: u64,
    pub quota: QuotaState,
}

/// Quota outlook as adjudicated by the ERP quota service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct QuotaState {
    pub remaining: i64,
    pub limit: i64,
    pub allowed: bool,
}

/// `rescue.diagnose` params. The diagnostics bundle is opaque to the
/// protocol (the model defines its shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct RescueDiagnoseParams {
    pub bundle: serde_json::Value,
}

/// `rescue.diagnose` result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct RescueDiagnoseResult {
    /// Structured diagnosis as produced by the model (opaque JSON).
    pub diagnosis: serde_json::Value,
    /// Model identifier that produced the diagnosis.
    pub model: String,
    /// RFC 3339 timestamp.
    pub generated_at: String,
}

/// `rescue.diagnose.progress` notification params.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct RescueDiagnoseProgressParams {
    /// Monotonic 1-based step counter.
    pub step: u32,
}

/// `gateway.info` result: capability descriptor served before
/// authentication (replaces the old `rescue_channel_not_configured` +
/// `missing` map dance).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct GatewayServiceInfo {
    /// Human-readable service identifier, e.g. `"gateway"`.
    pub service: String,
    /// Server version.
    pub version: String,
    /// Whether the rescue channel is configured and usable.
    pub rescue_ready: bool,
    /// Unset configuration keys that keep `rescue_ready` false.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    /// Deployment region descriptor (`"XX"` when unknown).
    pub region: String,
}

// ═══════════════════════════════════════════════════════════════
// Enrollment gateway (evernight)
// ═══════════════════════════════════════════════════════════════

/// `enrollment.mint` params (parity with the gateway's `EnrollRequest`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct EnrollmentMintParams {
    /// Free-form device label stored as the enrollment note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Explicit target server override (`ws(s)://…/api/ws`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_server: Option<String>,
    /// Parent signing-certificate flow: when present the gateway prepares
    /// the claims and the flasher signs locally with the key bound in the
    /// certificate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing: Option<SigningRequest>,
}

/// Parent-certificate mint selection inside [`EnrollmentMintParams`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct SigningRequest {
    pub parent_jti: String,
}

/// `enrollment.signing_cert.mint` params (parity with `SigningCertRequest`).
///
/// The protocol caps `ttl_days` at 3 years (1095 days): a stolen keystore
/// must die on its own within a bounded horizon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct SigningCertParams {
    /// Client-generated Ed25519 public key, base64url raw (the JWK `x`).
    /// The private half never leaves the flasher's keystore.
    pub public_key: String,
    /// Requested validity in days, window anchored at the operator's last
    /// login.
    pub ttl_days: i64,
}

/// `enrollment.mint` result, legacy gateway-signed path (parity with the
/// flasher's `EnrollmentMint`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct EnrollmentMinted {
    pub serial: String,
    pub node_id: String,
    pub enrollment_token: String,
    /// Epoch seconds.
    pub expires_at: i64,
    /// Epoch seconds.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_server: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// `enrollment.mint` result, parent-chained path (parity with the
/// flasher's `MintParentChained`): the gateway only assigns identifiers
/// and returns the unsigned JOSE signing input; the flasher signs it with
/// the key bound in its parent certificate before burning to media.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct EnrollmentMintParentChained {
    pub serial: String,
    pub node_id: String,
    /// Unsigned JOSE signing input (base64url `header.payload`).
    pub signing_input: String,
    pub parent_jti: String,
    /// Epoch seconds; clamped inside the parent certificate window.
    pub expires_at: i64,
    /// Epoch seconds.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_server: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// `enrollment.signing_cert.mint` result (parity with the flasher's
/// `ParentCertIssued`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct ParentCertIssued {
    /// The signed certificate JWT.
    pub certificate: String,
    pub jti: String,
    /// Not-before, epoch seconds (anchored at the operator's last login).
    pub nbf: i64,
    /// Expiry, epoch seconds.
    pub exp: i64,
}

/// `enrollment.status` result (parity with the flasher's
/// `EnrollmentStatus`; `status` stays a free string — `"pending"`,
/// `"activated"`, `"revoked"`, `"expired"` — until every deployment
/// agrees on the enum).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct EnrollmentStatus {
    pub serial: String,
    pub node_id: String,
    pub status: String,
    /// Epoch seconds.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activated_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activated_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<i64>,
    /// Epoch seconds.
    pub expires_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// `device.bootstrap` result: the one-shot payload a freshly flashed
/// device fetches with its enrollment token (parity with the host agent's
/// `BootstrapResponse`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "gateway.ts")]
pub struct DeviceBootstrap {
    pub node_id: String,
    /// Candidate evernight-server endpoints (`ws(s)://…/api/ws`).
    pub server_urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rescue_open_session_round_trip() {
        let params = RescueOpenSessionParams {
            ticket: "ticket-0001".into(),
        };
        let wire = serde_json::to_value(&params).unwrap();
        assert_eq!(wire, json!({"ticket": "ticket-0001"}));
        let back: RescueOpenSessionParams = serde_json::from_value(wire).unwrap();
        assert_eq!(back, params);

        let opened = RescueSessionOpened {
            expires_in: 3600,
            quota: QuotaState {
                remaining: 2,
                limit: 3,
                allowed: true,
            },
        };
        let wire = serde_json::to_value(&opened).unwrap();
        assert_eq!(wire["quota"]["remaining"], 2);
        assert!(wire.get("session_token").is_none());
        let back: RescueSessionOpened = serde_json::from_value(wire).unwrap();
        assert_eq!(back, opened);
    }

    #[test]
    fn diagnose_result_carries_opaque_diagnosis() {
        let result = RescueDiagnoseResult {
            diagnosis: json!({"root_cause": "psu undervoltage"}),
            model: "deepseek-flash".into(),
            generated_at: "2026-09-07T10:00:00Z".into(),
        };
        let wire = serde_json::to_value(&result).unwrap();
        assert_eq!(wire["diagnosis"]["root_cause"], "psu undervoltage");
        let back: RescueDiagnoseResult = serde_json::from_value(wire).unwrap();
        assert_eq!(back, result);
    }

    #[test]
    fn gateway_info_omits_empty_missing() {
        let info = GatewayServiceInfo {
            service: "gateway".into(),
            version: "0.1.0".into(),
            rescue_ready: true,
            missing: Vec::new(),
            region: "XX".into(),
        };
        let wire = serde_json::to_value(&info).unwrap();
        assert!(wire.get("missing").is_none());
        let back: GatewayServiceInfo = serde_json::from_value(wire).unwrap();
        assert_eq!(back, info);
    }

    #[test]
    fn enrollment_mint_params_match_gateway_shape() {
        // Parity witness for the legacy mint request body.
        let wire = json!({
            "name": "press-01",
            "signing": {"parent_jti": "cert-7"},
        });
        let params: EnrollmentMintParams = serde_json::from_value(wire).unwrap();
        assert_eq!(params.name.as_deref(), Some("press-01"));
        assert_eq!(params.target_server, None);
        assert_eq!(params.signing.as_ref().unwrap().parent_jti, "cert-7");
    }

    #[test]
    fn parent_chained_mint_round_trip() {
        let mint = EnrollmentMintParentChained {
            serial: "EVN-000123".into(),
            node_id: "node-abc".into(),
            signing_input: "aGVhZGVy.cGF5bG9hZA".into(),
            parent_jti: "cert-7".into(),
            expires_at: 1_800_000_000,
            created_at: 1_750_000_000,
            target_server: None,
            name: None,
        };
        let wire = serde_json::to_value(&mint).unwrap();
        assert!(wire.get("target_server").is_none());
        assert!(wire.get("name").is_none());
        let back: EnrollmentMintParentChained = serde_json::from_value(wire).unwrap();
        assert_eq!(back, mint);
    }

    #[test]
    fn device_bootstrap_parses_minimal_payload() {
        let wire = json!({"node_id": "node-abc", "server_urls": ["wss://api.evernight.celestia.world/api/ws"]});
        let bootstrap: DeviceBootstrap = serde_json::from_value(wire).unwrap();
        assert_eq!(bootstrap.device_secret, None);
        assert_eq!(bootstrap.server_urls.len(), 1);
    }

    #[test]
    fn enrollment_status_parses_legacy_optional_fields() {
        let wire = json!({
            "serial": "EVN-000123",
            "node_id": "node-abc",
            "status": "pending",
            "created_at": 1_750_000_000,
            "expires_at": 1_800_000_000
        });
        let status: EnrollmentStatus = serde_json::from_value(wire).unwrap();
        assert_eq!(status.activated_at, None);
        assert_eq!(status.activated_ip, None);
        assert_eq!(status.revoked_at, None);
    }
}
