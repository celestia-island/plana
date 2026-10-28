//! Declarative sibling-service HTTP client for celestia-island services.
//!
//! One [`SiblingService`] per target sibling per consumer process: endpoint
//! resolution (with ordered multi-endpoint failover), connection reuse,
//! layered timeouts, credential injection and no-proxy-by-default are all
//! owned by this facility, so a "call a sibling service" becomes a single
//! declarative chain.
//!
//! # Declaring a sibling
//!
//! ```no_run
//! use std::time::Duration;
//!
//! let arona = plana_sibling::SiblingService::new("arona")
//!     // endpoint sources, tried in declaration order:
//!     .endpoints_env("ARONA_SERVER_URLS")            // comma-separated list
//!     .endpoint_fixed("http://192.0.2.10:3002")      // deployment-pinned root
//!     .bearer_env("ARONA_SERVER_TOKEN")              // injected, never logged
//!     .build()?;
//!
//! let out: serde_json::Value = arona
//!     .call(plana_sibling::Method::POST, "/api/rpc", &serde_json::json!({"id": 1}))
//!     .timeout(Duration::from_secs(10))
//!     .retry(plana_sibling::Retry::idempotent(2))
//!     .send()
//!     .await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Semantics
//!
//! - **Failover**: endpoints are attempted in source declaration order.
//!   Connection failures, 5xx responses and timeouts advance to the next
//!   endpoint; 4xx is a business error and returns [`SiblingError::Status`]
//!   immediately without advancing.
//! - **Cross-endpoint retry is idempotency-gated**: GET/HEAD/OPTIONS/TRACE/
//!   PUT/DELETE fail over; a POST only fails over when it carries an
//!   idempotency key (`.idempotency_key(...)`). A non-idempotent POST is
//!   attempted exactly once against the first endpoint — retrying it against
//!   another endpoint could double-execute the operation.
//! - [`Retry::none()`] is a single pass through the endpoint list.
//!   [`Retry::idempotent(n)`] adds `n` extra attempts cycling back through
//!   the list (with two endpoints: A, B, A, ...). Both walk endpoints in
//!   order; attempts never hammer the same endpoint twice in a row when
//!   alternatives exist.
//! - **Default no_proxy**: the internally built `reqwest::Client` is always
//!   `.no_proxy()` unless `.via_proxy()` opts in. Sibling endpoints live on
//!   the internal network; new code can no longer forget proxy hygiene.
//! - **Layered timeouts**: connect/read/total are set at build time
//!   (`.connect_timeout` / `.read_timeout` / `.total_timeout`); the total
//!   deadline defaults to 10 s and can be overridden per call with
//!   `.timeout(...)`.
//! - **Credentials**: `.bearer_env(name)` / `.bearer_secret(...)` are read
//!   once at [`SiblingServiceBuilder::build`] (fail-fast) and stored in a
//!   redacting [`secrecy::SecretString`]; tokens never appear in errors,
//!   logs or `Debug` output.
//! - **Telemetry**: every attempt emits a `sibling_call` tracing span with
//!   `sibling`, `endpoint`, `method`, `path`, `result` and `elapsed_ms`.
//!   Metrics counters are reserved behind the feature-gated `metrics`
//!   module (see [`metrics`]) until the P2-1 observability baseline lands.

#![forbid(unsafe_op_in_unsafe_fn)]

mod builder;
mod call;
mod error;
mod provider;
mod retry;
mod service;

#[cfg(feature = "metrics")]
pub mod metrics;

pub use builder::{SiblingBuildError, SiblingServiceBuilder};
pub use error::{AttemptFailure, AttemptReason, SiblingError};
pub use http::Method;
pub use provider::EndpointProvider;
pub use retry::Retry;
pub use service::SiblingService;

/// Install the ring-based rustls crypto provider exactly once.
///
/// Required because the workspace `reqwest` is built with
/// `rustls-no-provider` (pure-Rust TLS): any `reqwest::Client` construction
/// panics without an installed provider. Another crate in the process may
/// already have installed one (e.g. a russh/hyper-rustls path), in which
/// case `install_default` returns `AlreadyInstalled` — tolerate that, since
/// any installed provider satisfies reqwest's requirement.
pub(crate) fn ensure_rustls_crypto_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if rustls::crypto::CryptoProvider::get_default().is_some() {
            return;
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
