//! Signed HTTP bridge to the ERP wallet service (feature `client`).
//!
//! # Line protocol (this module is the protocol's home until ERP ships it)
//!
//! Every wallet call is one `POST` to
//! `{base_url}`[`WALLET_SERVICE_PATH_PREFIX`]`/{method}` with a JSON body:
//!
//! ```json
//! {
//!   "method": "balance",
//!   "subject": {"owner": "user", "id": "0193…", "ext_key": null},
//!   "params": {},
//!   "ts": 1772900000
//! }
//! ```
//!
//! `method` is the trait method name (`balance`, `reserve`, `refund`,
//! `task_card`, `settle`, `credit`, `entries`); `params` carries the
//! method-specific arguments using the [`crate::types`] shapes.
//! `ts` is unix seconds.
//!
//! The request carries one header, [`SIGN_HEADER`] (`X-Wallet-Sign`),
//! holding the lowercase-hex HMAC-SHA256 over the canonical message
//!
//! ```text
//! method | subject.owner | subject.id | ts | <exact request body bytes>
//! ```
//!
//! (`|` is the literal separator byte; `subject.owner` is the snake_case
//! owner kind, `subject.id` the hyphenated lowercase UUID, `ts` its decimal
//! form). The ERP side verifies the same way with the shared service
//! secret, which is handed to [`ErpBridgeWallet::new`] per instance and
//! never persisted or logged by this crate.
//!
//! # Responses
//!
//! Success answers `{"data": <payload>}` with the payload shape from
//! [`crate::Wallet`] (e.g. a [`crate::WalletBalance`] for `balance`, an
//! integer for `reserve`). Failure answers
//! `{"error": {"code": <rpc code>, "message": "...", "have": …, "need": …,
//! "ref_id": …}}` where the optional structured fields ride along only for
//! the codes that need them; the raw error object maps back into
//! [`crate::WalletError`] by [`crate::error::WalletError::rpc_code`].
//!
//! # Failure mapping
//!
//! Transport faults (connect errors, timeouts, unparseable non-2xx
//! answers) become [`crate::WalletError::Unavailable`]; an unparsable 2xx
//! body or a success envelope under a failure status is
//! [`crate::WalletError::Internal`]; a parseable error envelope maps by its
//! code regardless of the HTTP status.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::error::WalletError;
use crate::types::{
    CreditReason, EntryPage, EntryQuery, ReserveOp, Settlement, TaskBudget, WalletBalance,
    WalletSubject,
};
use crate::wallet::Wallet;

/// Path prefix of the ERP wallet service API (single endpoint namespace,
/// one POST per method).
pub const WALLET_SERVICE_PATH_PREFIX: &str = "/api/wallet/service";

/// Header carrying the lowercase-hex HMAC-SHA256 request signature.
pub const SIGN_HEADER: &str = "X-Wallet-Sign";

/// Default request timeout; transport timeouts surface as
/// [`WalletError::Unavailable`].
pub const DEFAULT_TIMEOUT_SECS: u64 = 10;

type HmacSha256 = Hmac<Sha256>;

/// HMAC-SHA256 of `message` under `secret`, lowercase hex.
///
/// This is the exact algorithm both sides of the protocol run; the ERP
/// implementation must reproduce it byte-for-byte (see the RFC 4231 test
/// below for an independent vector).
pub fn sign_message(secret: &[u8], message: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(message);
    hex::encode(mac.finalize().into_bytes())
}

/// Canonical signature message: `method|owner|id|ts|` followed by the exact
/// request body bytes.
fn canonical_message(method: &str, subject: &WalletSubject, ts: i64, body: &[u8]) -> Vec<u8> {
    let mut message = format!(
        "{method}|{}|{}|{ts}|",
        owner_tag(subject.owner),
        subject.id.hyphenated()
    )
    .into_bytes();
    message.extend_from_slice(body);
    message
}

/// Snake_case owner tag on the wire (`OwnerKind` serde form).
fn owner_tag(owner: crate::types::OwnerKind) -> &'static str {
    match owner {
        crate::types::OwnerKind::User => "user",
        crate::types::OwnerKind::Group => "group",
        crate::types::OwnerKind::Org => "org",
    }
}

/// Request envelope for every wallet call.
#[derive(Serialize)]
struct RequestEnvelope<'a, P> {
    method: &'a str,
    subject: &'a WalletSubject,
    params: P,
    ts: i64,
}

/// Structured failure payload of the error envelope.
#[derive(Debug, Deserialize)]
struct RawError {
    code: i64,
    #[serde(default)]
    message: String,
    #[serde(default)]
    have: Option<i64>,
    #[serde(default)]
    need: Option<i64>,
    #[serde(default)]
    ref_id: Option<String>,
}

/// Response discrimination: `{"error": …}` wins over `{"data": …}`.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawResponse<T> {
    Err { error: RawError },
    Ok { data: T },
}

/// HTTP bridge implementing [`Wallet`] against the ERP wallet service.
///
/// The `Debug` rendering is manual and redacts the HMAC secret: a debug
/// print of a bridge must never surface the key.
#[derive(Clone)]
pub struct ErpBridgeWallet {
    http: Client,
    base_url: String,
    secret: Vec<u8>,
}

impl std::fmt::Debug for ErpBridgeWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErpBridgeWallet")
            .field("base_url", &self.base_url)
            .field("secret", &"[redacted]")
            .finish_non_exhaustive()
    }
}

impl ErpBridgeWallet {
    /// Bridge with the default [`DEFAULT_TIMEOUT_SECS`] timeout.
    ///
    /// `secret` is the shared service HMAC key (memory only — never log or
    /// persist it; tests use the `test-secret` placeholder).
    pub fn new(base_url: impl Into<String>, secret: impl Into<Vec<u8>>) -> Self {
        Self::with_timeout(base_url, secret, Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    }

    /// Bridge with an explicit request timeout.
    pub fn with_timeout(
        base_url: impl Into<String>,
        secret: impl Into<Vec<u8>>,
        timeout: Duration,
    ) -> Self {
        ensure_rustls_crypto_provider();
        let http = Client::builder()
            .timeout(timeout)
            .build()
            .expect("reqwest client with standard TLS provider builds");
        Self::with_http_client(base_url, secret, http)
    }

    /// Bridge around a caller-supplied client (timeout, proxying and
    /// connection pooling are then the caller's business).
    pub fn with_http_client(
        base_url: impl Into<String>,
        secret: impl Into<Vec<u8>>,
        http: Client,
    ) -> Self {
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            secret: secret.into(),
        }
    }

    /// One signed round trip: envelope → signature → POST → decode.
    async fn call<P: Serialize, T: DeserializeOwned>(
        &self,
        method: &'static str,
        s: &WalletSubject,
        params: P,
    ) -> Result<T, WalletError> {
        let envelope = RequestEnvelope {
            method,
            subject: s,
            params,
            ts: Utc::now().timestamp(),
        };
        let body = serde_json::to_vec(&envelope)
            .map_err(|e| WalletError::Internal(format!("serialize wallet request: {e}")))?;
        let signature = sign_message(
            &self.secret,
            &canonical_message(method, s, envelope.ts, &body),
        );
        let url = format!(
            "{}{WALLET_SERVICE_PATH_PREFIX}/{method}",
            self.base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .post(url)
            .header(SIGN_HEADER, signature)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| WalletError::Unavailable)?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|_| WalletError::Unavailable)?;
        match serde_json::from_slice::<RawResponse<T>>(&bytes) {
            Ok(RawResponse::Err { error }) => Err(map_remote_error(error)),
            Ok(RawResponse::Ok { data }) if status.is_success() => Ok(data),
            // A success payload under a failure status is protocol noise.
            Ok(RawResponse::Ok { .. }) => Err(WalletError::Internal(format!(
                "wallet service answered {status} with a success envelope"
            ))),
            // Unparseable body: service-side fault when the status already
            // says so, otherwise a decode failure on our side.
            Err(_) if !status.is_success() => Err(WalletError::Unavailable),
            Err(e) => Err(WalletError::Internal(format!(
                "unparsable wallet response ({status}): {e}"
            ))),
        }
    }
}

/// Map the remote error envelope back into the typed vocabulary by
/// [`WalletError::rpc_code`].
fn map_remote_error(error: RawError) -> WalletError {
    let RawError {
        code,
        message,
        have,
        need,
        ref_id,
    } = error;
    match code {
        -32060 => WalletError::Insufficient {
            have: have.unwrap_or(0),
            need: need.unwrap_or(0),
        },
        -32062 => WalletError::AlreadyReserved {
            ref_id: ref_id.unwrap_or_default(),
        },
        -32064 => WalletError::NotFound,
        -32002 => WalletError::Forbidden,
        -32066 => WalletError::AuthorityMoved,
        -32000 => WalletError::Unavailable,
        -32603 => WalletError::Internal(message),
        other => WalletError::Internal(format!("remote wallet error {other}: {message}")),
    }
}

/// Install the ring crypto provider exactly once, required because reqwest
/// is built `rustls-no-provider` in this workspace (same recipe as
/// plana-llm-provider; a provider installed by another crate is tolerated).
fn ensure_rustls_crypto_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if rustls::crypto::CryptoProvider::get_default().is_some() {
            return;
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Method-specific `params` payloads (the [`crate::types`] shapes cover
/// reserve and entries directly).
#[derive(Serialize)]
struct EmptyParams {}

#[derive(Serialize)]
struct RefParams<'a> {
    ref_id: &'a str,
}

#[derive(Serialize)]
struct RefundParams<'a> {
    ref_id: &'a str,
    amount: i64,
}

#[derive(Serialize)]
struct SettleParams<'a> {
    amount: i64,
    ref_id: &'a str,
}

#[derive(Serialize)]
struct CreditParams<'a> {
    amount: i64,
    reason: CreditReason,
    ref_type: &'a str,
    ref_id: &'a str,
}

#[async_trait]
impl Wallet for ErpBridgeWallet {
    async fn balance(&self, s: &WalletSubject) -> Result<WalletBalance, WalletError> {
        self.call("balance", s, EmptyParams {}).await
    }

    async fn reserve(&self, s: &WalletSubject, op: &ReserveOp) -> Result<i64, WalletError> {
        self.call("reserve", s, op).await
    }

    async fn refund(
        &self,
        s: &WalletSubject,
        ref_id: &str,
        amount: i64,
    ) -> Result<i64, WalletError> {
        self.call("refund", s, RefundParams { ref_id, amount })
            .await
    }

    async fn task_card(&self, s: &WalletSubject, ref_id: &str) -> Result<TaskBudget, WalletError> {
        self.call("task_card", s, RefParams { ref_id }).await
    }

    async fn settle(
        &self,
        s: &WalletSubject,
        amount: i64,
        ref_id: &str,
    ) -> Result<Settlement, WalletError> {
        self.call("settle", s, SettleParams { amount, ref_id })
            .await
    }

    async fn credit(
        &self,
        s: &WalletSubject,
        amount: i64,
        reason: CreditReason,
        ref_type: &str,
        ref_id: &str,
    ) -> Result<i64, WalletError> {
        self.call(
            "credit",
            s,
            CreditParams {
                amount,
                reason,
                ref_type,
                ref_id,
            },
        )
        .await
    }

    async fn entries(&self, s: &WalletSubject, q: &EntryQuery) -> Result<EntryPage, WalletError> {
        self.call("entries", s, q).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OwnerKind, SettlementOutcome, Unit};
    use uuid::Uuid;
    use wiremock::matchers::{method as match_method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Placeholder key for every test in this crate — a real secret never
    /// enters the tree (workspace red line 1).
    const TEST_SECRET: &[u8] = b"test-secret";

    fn subject() -> WalletSubject {
        WalletSubject {
            owner: OwnerKind::User,
            id: Uuid::parse_str("01933e52-7c7a-7113-bf13-9a01a3d1f6a7").unwrap(),
            ext_key: Some("user@example.com".into()),
        }
    }

    /// Independent vector: RFC 4231 test case 1 (key = 0x0b × 20,
    /// message = "Hi There"). Proves the signer against the published
    /// HMAC-SHA256 definition, not against itself.
    #[test]
    fn signature_matches_rfc_4231_vector() {
        let signature = sign_message(&[0x0bu8; 20], b"Hi There");
        assert_eq!(
            signature,
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn signature_is_deterministic_and_input_sensitive() {
        let a = sign_message(TEST_SECRET, b"balance|user|0193|1|{}");
        let again = sign_message(TEST_SECRET, b"balance|user|0193|1|{}");
        assert_eq!(a, again);
        // Any input change (ts, body, method, key) must move the signature.
        assert_ne!(a, sign_message(TEST_SECRET, b"balance|user|0193|2|{}"));
        assert_ne!(a, sign_message(TEST_SECRET, b"reserve|user|0193|1|{}"));
        assert_ne!(a, sign_message(b"test-secret-2", b"balance|user|0193|1|{}"));
    }

    #[test]
    fn canonical_message_is_method_owner_id_ts_body() {
        let s = subject();
        let message = canonical_message("balance", &s, 1_772_900_000, b"{}");
        assert_eq!(
            String::from_utf8(message).unwrap(),
            "balance|user|01933e52-7c7a-7113-bf13-9a01a3d1f6a7|1772900000|{}"
        );
    }

    /// Full round trip over the wire: the recorded request must carry the
    /// path, envelope shape and a signature the server-side recipe
    /// reproduces from the shared secret — this test is the ERP verifier
    /// dry-run.
    #[tokio::test]
    async fn balance_round_trip_signs_and_decodes() {
        let server = MockServer::start().await;
        Mock::given(match_method("POST"))
            .and(path(format!("{WALLET_SERVICE_PATH_PREFIX}/balance")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"available": 1234, "unit": "points"}
            })))
            .mount(&server)
            .await;

        let bridge = ErpBridgeWallet::new(server.uri(), TEST_SECRET);
        let balance = bridge.balance(&subject()).await.unwrap();
        assert_eq!(
            balance,
            WalletBalance {
                available: 1234,
                unit: Unit::Points,
            }
        );

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        let signature = request
            .headers
            .get(SIGN_HEADER)
            .expect("signature header present")
            .to_str()
            .unwrap();
        // Reproduce the server-side verification from the raw body bytes.
        let envelope: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
        assert_eq!(envelope["method"], "balance");
        assert_eq!(envelope["subject"]["owner"], "user");
        let ts = envelope["ts"].as_i64().unwrap();
        let expected = sign_message(
            TEST_SECRET,
            &canonical_message("balance", &subject(), ts, &request.body),
        );
        assert_eq!(signature, expected);
    }

    #[tokio::test]
    async fn error_envelope_maps_back_to_typed_errors() {
        let server = MockServer::start().await;
        Mock::given(path(format!("{WALLET_SERVICE_PATH_PREFIX}/reserve")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "error": {
                    "code": -32060,
                    "message": "quota exceeded: have 40, need 100",
                    "have": 40,
                    "need": 100
                }
            })))
            .mount(&server)
            .await;

        let bridge = ErpBridgeWallet::new(server.uri(), TEST_SECRET);
        let err = bridge
            .reserve(
                &subject(),
                &ReserveOp {
                    ref_id: "task-1".into(),
                    amount: 100,
                },
            )
            .await
            .unwrap_err();
        assert_eq!(
            err,
            WalletError::Insufficient {
                have: 40,
                need: 100
            }
        );
    }

    #[tokio::test]
    async fn settle_round_trip_decodes_the_outcome() {
        let server = MockServer::start().await;
        Mock::given(path(format!("{WALLET_SERVICE_PATH_PREFIX}/settle")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"outcome": "clamped", "deducted": 50}
            })))
            .mount(&server)
            .await;
        let bridge = ErpBridgeWallet::new(server.uri(), TEST_SECRET);
        let out = bridge.settle(&subject(), 80, "task-1").await.unwrap();
        assert_eq!(out.outcome, SettlementOutcome::Clamped);
        assert_eq!(out.deducted, 50);
    }

    #[tokio::test]
    async fn http_5xx_with_html_body_is_unavailable() {
        let server = MockServer::start().await;
        Mock::given(wiremock::matchers::any())
            .respond_with(ResponseTemplate::new(503).set_body_string("<html>boom</html>"))
            .mount(&server)
            .await;
        let bridge = ErpBridgeWallet::new(server.uri(), TEST_SECRET);
        let err = bridge.balance(&subject()).await.unwrap_err();
        assert_eq!(err, WalletError::Unavailable);
    }

    #[tokio::test]
    async fn dead_endpoint_is_unavailable() {
        // Bind, read the port, drop the listener: deterministic connect
        // refusal (same trick as plana-sibling's tests).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let bridge = ErpBridgeWallet::new(format!("http://127.0.0.1:{port}"), TEST_SECRET);
        let err = bridge.balance(&subject()).await.unwrap_err();
        assert_eq!(err, WalletError::Unavailable);
    }
}
