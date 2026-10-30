//! The per-call builder ([`SiblingCall`]) and the attempt loop that realizes
//! the failover / retry / timeout semantics.

use std::time::{Duration, Instant};

use secrecy::ExposeSecret;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::Instrument as _;

use crate::error::{AttemptFailure, AttemptReason, SiblingError, truncate_body};
use crate::retry::{Retry, method_is_idempotent};
use crate::service::SiblingService;

/// One in-flight call — returned by
/// [`SiblingService::call`](crate::SiblingService::call), finished by
/// [`send`](Self::send) / [`send_bytes`](Self::send_bytes).
pub struct SiblingCall<'svc, 'req, B: Serialize + ?Sized> {
    pub(crate) service: &'svc SiblingService,
    pub(crate) method: http::Method,
    pub(crate) path: String,
    pub(crate) body: Option<&'req B>,
    pub(crate) query_pairs: Vec<(String, String)>,
    pub(crate) retry: Retry,
    pub(crate) total_timeout: Option<Duration>,
    pub(crate) idempotency_key: Option<String>,
}

impl<B: Serialize + ?Sized> SiblingCall<'_, '_, B> {
    /// Override the call's total wall-clock deadline (shared across all
    /// failover attempts — the worst case is not attempts × deadline);
    /// connect/read stay at their build-time values.
    #[must_use]
    pub fn timeout(mut self, d: Duration) -> Self {
        self.total_timeout = Some(d);
        self
    }

    /// Append one query parameter (applied to every attempt; values are
    /// URL-encoded by the transport). Repeatable.
    #[must_use]
    pub fn query_pair(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_pairs.push((key.into(), value.into()));
        self
    }

    /// Set the cross-endpoint retry policy (see [`Retry`]).
    #[must_use]
    pub fn retry(mut self, retry: Retry) -> Self {
        self.retry = retry;
        self
    }

    /// Attach an idempotency key (`Idempotency-Key` header). This is the
    /// caller's promise the operation is safe to replay, so a POST carrying
    /// one gains the same failover rights as GET (a POST without one still
    /// advances across connect-phase failures — those prove the request
    /// reached no server — but never across delivered/unknown failures).
    #[must_use]
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Execute and return the raw response body (any 2xx/3xx status).
    ///
    /// # Errors
    /// See [`SiblingError`] — the five failure classes with context.
    pub async fn send_bytes(self) -> Result<Vec<u8>, SiblingError> {
        let SiblingCall {
            service,
            method,
            path,
            body,
            query_pairs,
            retry,
            total_timeout,
            idempotency_key,
        } = self;

        let name = service.name().to_string();
        let payload: Option<Vec<u8>> = if is_bodyless(&method) {
            None
        } else {
            match body {
                Some(b) => match serde_json::to_vec(b) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        return Err(SiblingError::Decode {
                            service: name,
                            cause: format!("request body: {e}"),
                        });
                    }
                },
                None => None,
            }
        };

        execute(
            service,
            &name,
            &method,
            &path,
            payload.as_deref(),
            query_pairs,
            retry,
            total_timeout,
            idempotency_key.as_deref(),
        )
        .await
    }

    /// Execute and decode the JSON response body into `T`.
    ///
    /// # Errors
    /// See [`SiblingError`]; non-JSON bodies surface as
    /// [`SiblingError::Decode`] — use [`send_bytes`](Self::send_bytes) for
    /// non-JSON endpoints.
    pub async fn send<T: DeserializeOwned>(self) -> Result<T, SiblingError> {
        let name = self.service.name().to_string();
        let bytes = self.send_bytes().await?;
        serde_json::from_slice(&bytes).map_err(|e| SiblingError::Decode {
            service: name,
            cause: format!("response body: {e}"),
        })
    }
}

/// GET/HEAD never carry a JSON body (pass `&()` in `call`).
fn is_bodyless(method: &http::Method) -> bool {
    method == http::Method::GET || method == http::Method::HEAD
}

/// Internal per-attempt failure (bodies kept only while the outcome is
/// still being classified — records strip them).
enum AttemptFail {
    /// The connection was never established (refused, DNS, TLS handshake,
    /// connect timeout) or the request never left the client (URL build
    /// failure): nothing was delivered, so advancing to the next endpoint
    /// is safe even for a non-idempotent call — replay cannot
    /// double-execute what no server received.
    Connect(String),
    /// Transport failure at or after delivery (mid-body reset, response
    /// read error): the endpoint may already have processed the request,
    /// so a non-idempotent call stops at this endpoint.
    Unreachable(String),
    Timeout(Duration),
    Status(u16, String),
}

impl AttemptFail {
    fn to_reason(&self) -> AttemptReason {
        match self {
            Self::Connect(cause) | Self::Unreachable(cause) => {
                AttemptReason::Unreachable(cause.clone())
            }
            Self::Timeout(after) => AttemptReason::Timeout(*after),
            Self::Status(code, _) => AttemptReason::Status(*code),
        }
    }

    /// True when this failure proves the request reached no server — the
    /// one condition under which a NON-idempotent call may advance to the
    /// next endpoint.
    fn never_delivered(&self) -> bool {
        matches!(self, Self::Connect(_))
    }
}

/// A non-idempotent call that stops on a delivered/unknown failure must
/// report the specific failure class — "all endpoints down" would be false
/// when the never-delivered endpoints were skipped past.
fn specific_error(name: &str, endpoint: &str, fail: AttemptFail) -> SiblingError {
    match fail {
        AttemptFail::Connect(cause) | AttemptFail::Unreachable(cause) => {
            SiblingError::Unreachable {
                service: name.to_string(),
                endpoint: endpoint.to_string(),
                cause,
            }
        }
        AttemptFail::Timeout(after) => SiblingError::Timeout {
            service: name.to_string(),
            after,
        },
        AttemptFail::Status(code, body) => SiblingError::Status {
            service: name.to_string(),
            code,
            body,
        },
    }
}

/// Join an endpoint root and a call path into a validated absolute URL.
/// `endpoint` may carry a trailing slash; `path` may omit the leading one.
/// An **empty** path dials the endpoint verbatim (no slash appended) — the
/// shape for providers that hand over full URLs (path included).
fn build_url(endpoint: &str, path: &str) -> Result<String, String> {
    let base = endpoint.trim_end_matches('/');
    let full = if path.is_empty() {
        base.to_string()
    } else if path.starts_with('/') {
        format!("{base}{path}")
    } else {
        format!("{base}/{path}")
    };
    reqwest::Url::parse(&full).map_err(|e| format!("invalid endpoint URL `{full}`: {e}"))?;
    Ok(full)
}

#[allow(clippy::too_many_arguments)]
async fn execute(
    service: &SiblingService,
    name: &str,
    method: &http::Method,
    path: &str,
    payload: Option<&[u8]>,
    query_pairs: Vec<(String, String)>,
    retry: Retry,
    total_override: Option<Duration>,
    idempotency_key: Option<&str>,
) -> Result<Vec<u8>, SiblingError> {
    let endpoints = service.resolve_endpoints();
    if endpoints.is_empty() {
        return Err(SiblingError::AllEndpointsDown {
            service: name.to_string(),
            tried: Vec::new(),
        });
    }
    let idempotent = method_is_idempotent(method, idempotency_key.is_some());
    let effective_total = total_override.unwrap_or_else(|| service.total_timeout());
    let max_attempts = match (retry, idempotent) {
        // One ordered pass across the list.
        (Retry::None, true) => endpoints.len(),
        // Non-idempotent: one ordered pass, same budget — but the loop
        // below stops at the FIRST endpoint whose failure class does not
        // prove non-delivery, so the pass is only ever consumed by
        // never-delivered (connect-phase) failures.
        (Retry::None, false) | (Retry::Idempotent { .. }, false) => endpoints.len(),
        // Full pass plus cycling extra attempts.
        (Retry::Idempotent { max_retries }, true) => endpoints.len() + max_retries as usize,
    };
    if let (Retry::Idempotent { max_retries }, false) = (retry, idempotent)
        && max_retries > 0
    {
        tracing::warn!(
            sibling = name,
            method = %method,
            "retry policy idempotent({max_retries}) has no effect on a \
             non-idempotent request; advancing only across never-delivered \
             (connect-phase) failures, no cycling"
        );
    }

    // The total timeout is a call-level wall-clock deadline: every failover
    // attempt draws from the same budget (worst case is NOT attempts ×
    // total). Individual attempts stay bounded by the build-time
    // connect/read layers plus whatever is left of this deadline.
    let call = async {
        let mut tried: Vec<AttemptFailure> = Vec::new();
        for attempt in 0..max_attempts {
            let endpoint = endpoints[attempt % endpoints.len()].clone();
            let started = Instant::now();
            let outcome = {
                let span = tracing::info_span!(
                    "sibling_call",
                    sibling = name,
                    endpoint = %endpoint,
                    method = %method,
                    path = path,
                    result = tracing::field::Empty,
                    elapsed_ms = tracing::field::Empty,
                );
                async {
                    let res = do_attempt(
                        service,
                        method,
                        path,
                        payload,
                        &query_pairs,
                        idempotency_key,
                        &endpoint,
                    )
                    .await;
                    let elapsed_ms = started.elapsed().as_millis() as u64;
                    let result_text = match &res {
                        Ok(_) => "ok".to_string(),
                        Err(fail) => match fail {
                            AttemptFail::Connect(cause) => {
                                format!("connect (never delivered): {cause}")
                            }
                            AttemptFail::Unreachable(cause) => format!("unreachable: {cause}"),
                            AttemptFail::Timeout(_) => "timeout".to_string(),
                            AttemptFail::Status(code, _) => format!("HTTP {code}"),
                        },
                    };
                    let cur = tracing::Span::current();
                    cur.record("result", result_text.as_str());
                    cur.record("elapsed_ms", elapsed_ms);
                    // One event per attempt so default tracing-subscriber
                    // setups (which print events, not span lifecycles) still
                    // see the per-endpoint outcome; the span carries the same
                    // fields for structured consumers.
                    tracing::info!(
                        result = result_text.as_str(),
                        elapsed_ms,
                        "sibling_call_done"
                    );
                    res
                }
                .instrument(span)
                .await
            };

            match outcome {
                Ok(bytes) => return Ok(bytes),
                // 4xx is a business error: terminal on every endpoint, by design.
                Err(AttemptFail::Status(code, body)) if (400..500).contains(&code) => {
                    return Err(SiblingError::Status {
                        service: name.to_string(),
                        code,
                        body,
                    });
                }
                Err(fail) => {
                    tried.push(AttemptFailure {
                        endpoint: endpoint.clone(),
                        reason: fail.to_reason(),
                    });
                    // A non-idempotent call advances ONLY on a failure that
                    // proves the request reached no server (connect-phase);
                    // a delivered/unknown failure (timeout after send, 5xx,
                    // mid-body reset) stops here — replaying it elsewhere
                    // could double-execute.
                    if !idempotent && !fail.never_delivered() {
                        return Err(specific_error(name, &endpoint, fail));
                    }
                }
            }
        }

        // Exhaustion implies every endpoint was covered: the cycling budget
        // is `len + retries`, `Retry::None` is exactly `len`, and a
        // non-idempotent pass only ever consumed never-delivered failures.
        Err(SiblingError::AllEndpointsDown {
            service: name.to_string(),
            tried,
        })
    };

    match tokio::time::timeout(effective_total, call).await {
        Ok(result) => result,
        // The call-level deadline expired (an in-flight attempt's span ends
        // without a recorded result; the call outcome is reported here).
        Err(_) => Err(SiblingError::Timeout {
            service: name.to_string(),
            after: effective_total,
        }),
    }
}

async fn do_attempt(
    service: &SiblingService,
    method: &http::Method,
    path: &str,
    payload: Option<&[u8]>,
    query_pairs: &[(String, String)],
    idempotency_key: Option<&str>,
    endpoint: &str,
) -> Result<Vec<u8>, AttemptFail> {
    let started = Instant::now();
    // A URL that fails to build never leaves the client: nothing was
    // delivered, same as a refused connection.
    let url = build_url(endpoint, path).map_err(AttemptFail::Connect)?;

    let mut req = service.client().request(method.clone(), url);
    for (key, value) in query_pairs {
        req = req.query(&[(key.as_str(), value.as_str())]);
    }
    if let Some(source) = service.token() {
        // Static tokens resolve at build; dynamic providers resolve here so
        // a rotated credential applies from this attempt on.
        if let Some(token) = source.token_for_attempt() {
            req = req.header(
                http::header::AUTHORIZATION,
                format!("Bearer {}", token.expose_secret()),
            );
        }
    }
    if let Some(key) = idempotency_key {
        req = req.header("Idempotency-Key", key);
    }
    if let Some(bytes) = payload {
        // Request bodies are always `serde_json::to_vec` output (see
        // `send_bytes`), so label them as JSON: axum-backed siblings
        // (e.g. arona's `/api/rpc`) reject an unlabeled body with 415,
        // which callers surface as "unreachable" — the 2026-10-29
        // empty-roster outage class.
        req = req
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(bytes.to_vec());
    }
    // No per-request total timeout here: the call-level deadline
    // (`tokio::time::timeout` around the whole attempt loop) is the total
    // layer; the connect/read layers come from the build-time client config.

    let resp = req.send().await.map_err(|e| {
        // Connect-phase first: a connect timeout is BOTH is_connect() and
        // is_timeout(), and the request provably reached no server — the
        // one error class a non-idempotent call may replay elsewhere.
        // Everything else on the send path (read timeout after delivery,
        // reset mid-write) is delivered-or-unknown: terminal for a
        // non-idempotent call.
        if e.is_connect() {
            AttemptFail::Connect(e.to_string())
        } else if e.is_timeout() {
            AttemptFail::Timeout(started.elapsed())
        } else {
            AttemptFail::Unreachable(e.to_string())
        }
    })?;

    let status = resp.status().as_u16();
    if (400..600).contains(&status) {
        let body = resp.bytes().await.map(|b| b.to_vec()).map_err(|e| {
            if e.is_timeout() {
                AttemptFail::Timeout(started.elapsed())
            } else {
                AttemptFail::Unreachable(e.to_string())
            }
        })?;
        return Err(AttemptFail::Status(status, truncate_body(&body)));
    }
    let body = resp.bytes().await.map(|b| b.to_vec()).map_err(|e| {
        if e.is_timeout() {
            AttemptFail::Timeout(started.elapsed())
        } else {
            AttemptFail::Unreachable(e.to_string())
        }
    })?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_url_joins_roots_and_paths() {
        assert_eq!(
            build_url("http://192.0.2.1:3002", "/api/rpc").unwrap(),
            "http://192.0.2.1:3002/api/rpc"
        );
        assert_eq!(
            build_url("http://192.0.2.1:3002/", "api/rpc").unwrap(),
            "http://192.0.2.1:3002/api/rpc"
        );
        assert_eq!(
            build_url("http://192.0.2.1:3002", "api/x?y=1").unwrap(),
            "http://192.0.2.1:3002/api/x?y=1"
        );
        // Empty path dials the endpoint verbatim (full-URL providers) —
        // no trailing slash appended.
        assert_eq!(
            build_url("http://192.0.2.1:3008/api/ws", "").unwrap(),
            "http://192.0.2.1:3008/api/ws"
        );
        assert!(build_url("::::", "/x").is_err());
    }

    #[test]
    fn get_and_head_are_bodyless() {
        assert!(is_bodyless(&http::Method::GET));
        assert!(is_bodyless(&http::Method::HEAD));
        assert!(!is_bodyless(&http::Method::POST));
    }
}
