//! Error taxonomy for sibling calls.
//!
//! Five classes ([`SiblingError`]), each carrying enough context to answer
//! "which sibling, which endpoint, what happened" without re-running the
//! call — the whole point is to eliminate the "Option swallowed every error
//! class" pattern found in the audit (arch-review-20261026 T3).

use std::fmt;
use std::time::Duration;

/// Maximum response-body bytes quoted inside [`SiblingError::Status`] —
/// keeps error strings (and log lines) bounded for large error pages.
pub(crate) const MAX_BODY_BYTES: usize = 2048;

/// One failed attempt inside a [`SiblingError::AllEndpointsDown`] report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptFailure {
    /// Endpoint base URL that was attempted.
    pub endpoint: String,
    /// Why the attempt failed.
    pub reason: AttemptReason,
}

/// Why a single attempt failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptReason {
    /// Could not establish/use the connection (refused, reset, DNS, ...).
    Unreachable(String),
    /// A timeout layer (connect/read/total) cut the attempt short.
    Timeout(Duration),
    /// The endpoint answered with a 5xx.
    Status(u16),
}

impl fmt::Display for AttemptReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(cause) => write!(f, "unreachable: {cause}"),
            Self::Timeout(after) => write!(f, "timeout after {after:?}"),
            Self::Status(code) => write!(f, "HTTP {code}"),
        }
    }
}

impl fmt::Display for AttemptFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.endpoint, self.reason)
    }
}

/// Truncate a response body for quoting inside error variants (lossy UTF-8).
pub(crate) fn truncate_body(bytes: &[u8]) -> String {
    if bytes.len() <= MAX_BODY_BYTES {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let mut head = String::from_utf8_lossy(&bytes[..MAX_BODY_BYTES]).into_owned();
    head.push_str(&format!("…[{} bytes total]", bytes.len()));
    head
}

/// The five sibling-call failure classes. Every variant names the sibling and
/// carries per-class context; bearer tokens never appear anywhere here.
#[derive(Debug, thiserror::Error)]
pub enum SiblingError {
    /// The endpoint could not be reached (connection refused/reset, DNS,
    /// TLS, mid-body transport failure). Reported for the failing endpoint
    /// when a non-idempotent call stops there — which today is only a
    /// delivered/unknown failure (read timeout, mid-body reset): a
    /// connect-phase failure never stops a non-idempotent call, it advances
    /// to the next endpoint, and exhausting the list reports
    /// [`SiblingError::AllEndpointsDown`] instead.
    #[error("sibling `{service}` endpoint `{endpoint}` unreachable: {cause}")]
    Unreachable {
        service: String,
        endpoint: String,
        cause: String,
    },
    /// A timeout layer fired; `after` is how long the attempt ran.
    #[error("sibling `{service}` request timed out after {after:?}")]
    Timeout { service: String, after: Duration },
    /// The endpoint answered 4xx (business error — no failover) or 5xx on a
    /// non-idempotent call that does not advance. `body` is the (truncated)
    /// response body.
    #[error("sibling `{service}` returned HTTP {code}: {body}")]
    Status {
        service: String,
        code: u16,
        body: String,
    },
    /// JSON encode (request body) or decode (response body) failed; `cause`
    /// is prefixed `request body:` or `response body:` to tell the halves
    /// apart.
    #[error("sibling `{service}` JSON encode/decode failed: {cause}")]
    Decode { service: String, cause: String },
    /// Every endpoint available for this call was tried (or the list was
    /// empty) and none succeeded; `tried` is the per-attempt failure record
    /// in attempt order.
    #[error("sibling `{service}` all endpoints down, tried {tried:?}")]
    AllEndpointsDown {
        service: String,
        tried: Vec<AttemptFailure>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_body_keeps_short_bodies_whole() {
        assert_eq!(truncate_body(b"nope"), "nope");
    }

    #[test]
    fn truncate_body_marks_long_bodies() {
        // Well past the cap so truncation actually shortens the payload.
        let long = vec![b'a'; MAX_BODY_BYTES * 3];
        let out = truncate_body(&long);
        assert!(out.contains("bytes total"));
        assert!(out.len() < long.len());
        assert!(out.starts_with(&"a".repeat(MAX_BODY_BYTES)));
    }
}
