//! Cross-endpoint retry policy.

/// Retry policy for a sibling call.
///
/// Failover across endpoints is idempotency-gated regardless of the policy:
/// idempotent methods walk the endpoint list in order; a POST without an
/// idempotency key also walks the list but only advances past a failure
/// that proves non-delivery (connect-phase: refused, DNS, TLS handshake,
/// connect timeout) — a delivered/unknown failure (read timeout, 5xx,
/// mid-body reset) stops the call at that endpoint under every policy,
/// because replaying it elsewhere could double-execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Retry {
    /// One pass through the endpoint list, no re-attempts. For an idempotent
    /// method this still fails over (list order is respected once).
    #[default]
    None,
    /// A full pass through the endpoint list plus `max_retries` extra
    /// attempts, cycling back to the first endpoint after the last (with
    /// endpoints `[A, B]` and `idempotent(2)`: A, B, A, B). `idempotent(0)`
    /// is equivalent to [`Retry::None`].
    Idempotent { max_retries: u32 },
}

impl Retry {
    /// Single pass, no retries.
    #[must_use]
    pub fn none() -> Self {
        Self::None
    }

    /// Full pass plus `max_retries` cycling extra attempts.
    /// `idempotent(0)` normalizes to [`Retry::None`].
    #[must_use]
    pub fn idempotent(max_retries: u32) -> Self {
        if max_retries == 0 {
            Self::None
        } else {
            Self::Idempotent { max_retries }
        }
    }
}

/// HTTP methods that are idempotent by definition.
///
/// POST joins this set only when it carries an idempotency key; the key is
/// the caller's promise that replay is safe (and downstream services rely on
/// it to deduplicate).
pub(crate) fn method_is_idempotent(method: &http::Method, has_idempotency_key: bool) -> bool {
    matches!(
        method.as_str(),
        "GET" | "HEAD" | "OPTIONS" | "TRACE" | "PUT" | "DELETE"
    ) || (method == http::Method::POST && has_idempotency_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_methods_are_idempotent() {
        for m in ["GET", "HEAD", "OPTIONS", "TRACE", "PUT", "DELETE"] {
            assert!(method_is_idempotent(
                &http::Method::from_bytes(m.as_bytes()).unwrap(),
                false
            ));
        }
    }

    #[test]
    fn post_needs_an_idempotency_key() {
        let post = http::Method::POST;
        assert!(!method_is_idempotent(&post, false));
        assert!(method_is_idempotent(&post, true));
        assert!(!method_is_idempotent(&http::Method::PATCH, true));
    }

    #[test]
    fn idempotent_zero_equals_none() {
        assert_eq!(Retry::idempotent(0), Retry::none());
    }
}
