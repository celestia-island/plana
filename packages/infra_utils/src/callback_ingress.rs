//! `X-Celestia-Signature` ingress verification — the one implementation.
//!
//! The workspace has three services that receive signed callbacks (arona for model
//! results, evernight's node for MCP execution, celestia-erp for payment notices), and
//! each grew its own verifier. By 2026-11-12 they had drifted: one accepted a 1 MiB
//! body while another capped at 64 KiB, one collected every `v1=` candidate for key
//! rotation while another kept only the last, and one returned a three-word rejection
//! vocabulary where another had six. Same header, same HMAC construction, different
//! answers — the contract was in people's heads.
//!
//! This module is that contract as code. Wire format (see the workspace contract doc):
//!
//! ```text
//! X-Celestia-Signature: t=<unix seconds>,v1=<hex hmac_sha256(secret, "{t}.{raw_body}")>
//! ```
//!
//! Rejections are a closed set — callers map them onto their own verdict vocabulary but
//! never invent a new one:
//!
//! | [`Rejection`] | wire code | meaning |
//! |---|---|---|
//! | [`Rejection::Missing`] | `signature_missing` | no header at all |
//! | [`Rejection::Malformed`] | `signature_malformed` | header present but unparsable |
//! | [`Rejection::Mismatch`] | `signature_mismatch` | MAC does not match any candidate |
//! | [`Rejection::Oversized`] | `body_too_large` | raw body exceeds the caller's cap (checked before the MAC) |
//! | [`Rejection::Stale`] | `signature_expired` | timestamp outside the skew window |
//!
//! Two details are load-bearing and easy to get wrong:
//!
//! * **The timestamp window is checked before the MAC** — a stale header must never burn
//!   an HMAC computation, and more importantly must never be reported as a mismatch
//!   (which would tell an attacker their MAC was wrong rather than that they were late).
//! * **Every `v1=` candidate is tried** — key rotation needs the old and new secret to
//!   overlap, so a header may legitimately carry more than one. Collect, don't pick.

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Header carrying the signature.
pub const SIGNATURE_HEADER: &str = "x-celestia-signature";

/// Default tolerance for clock skew and delivery delay, in seconds.
pub const DEFAULT_MAX_SKEW_SECS: i64 = 300;

/// Default cap on the raw body, in bytes. Callers that accept larger payloads must say
/// so explicitly — the cap exists so an unauthenticated body cannot be arbitrarily big.
pub const DEFAULT_MAX_BODY_BYTES: usize = 64 * 1024;

/// Why a callback was rejected. Closed set; see the module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// No signature header on the request.
    Missing,
    /// The header is present but not `t=<unix>,v1=<hex>[,v1=<hex>...]`.
    Malformed,
    /// The timestamp is outside the accepted window.
    Stale,
    /// No candidate matched the MAC.
    Mismatch,
    /// The raw body exceeded the caller's cap. Rejected **before** the signature is
    /// judged: an oversized unauthenticated body must not reach the HMAC at all.
    Oversized,
}

impl Rejection {
    /// The wire code this rejection maps to. Stable strings: other services log them.
    #[must_use]
    pub fn wire_code(self) -> &'static str {
        match self {
            Rejection::Missing => "signature_missing",
            Rejection::Malformed => "signature_malformed",
            Rejection::Stale => "signature_expired",
            Rejection::Mismatch => "signature_mismatch",
            Rejection::Oversized => "body_too_large",
        }
    }
}

/// A verified callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verified {
    /// The `t=` value that was signed.
    pub timestamp: i64,
}

/// Verify a callback signature.
///
/// * `secret` — the shared secret for this (provider, source) pair, raw bytes.
/// * `header` — the `X-Celestia-Signature` value, or `None` when absent.
/// * `body` — the **raw** request body, exactly as received.
/// * `now_unix` — the current time; a parameter so callers can test without a clock.
/// * `max_skew_secs` — accepted absolute difference between `t` and `now_unix`.
///
/// The window is checked before the MAC on purpose: a late delivery is a logging
/// problem, a bad MAC is an attack, and conflating them both hides the attack and
/// wastes a HMAC per late request.
pub fn verify(
    secret: &[u8],
    header: Option<&str>,
    body: &[u8],
    now_unix: i64,
    max_skew_secs: i64,
) -> Result<Verified, Rejection> {
    verify_capped(
        secret,
        header,
        body,
        now_unix,
        max_skew_secs,
        DEFAULT_MAX_BODY_BYTES,
    )
}

/// Same as [`verify`], with an explicit cap on the raw body.
///
/// Callers that legitimately accept larger payloads must say so here — a payment notice
/// is not the same size as a model-result callback, and quietly inheriting the 64 KiB
/// default is how one service ended up with 1 MiB while another had 64 KiB.
pub fn verify_capped(
    secret: &[u8],
    header: Option<&str>,
    body: &[u8],
    now_unix: i64,
    max_skew_secs: i64,
    max_body_bytes: usize,
) -> Result<Verified, Rejection> {
    // A zero-length secret is a deployment error with a security consequence: HMAC
    // accepts any key length, so `new_from_slice(b"")` succeeds and every caller in the
    // world can compute the "signature". Fail closed rather than pretend it is a MAC.
    if secret.is_empty() {
        return Err(Rejection::Mismatch);
    }

    if body.len() > max_body_bytes {
        return Err(Rejection::Oversized);
    }

    let Some(header) = header else {
        return Err(Rejection::Missing);
    };

    let (timestamp, candidates) = parse_header(header)?;

    if (now_unix - timestamp).abs() > max_skew_secs {
        return Err(Rejection::Stale);
    }

    let mut signed = Vec::with_capacity(12 + body.len());
    signed.extend_from_slice(timestamp.to_string().as_bytes());
    signed.push(b'.');
    signed.extend_from_slice(body);

    let mut matched = false;

    for candidate in candidates {
        // `secret` is non-empty here (checked above) and HMAC accepts any key length, so
        // this cannot fail in practice.
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret) else {
            return Err(Rejection::Mismatch);
        };
        mac.update(&signed);

        // `verify_slice` is constant-time and rejects on length mismatch.
        if mac.verify_slice(&candidate).is_ok() {
            matched = true;
            break;
        }
    }

    if matched {
        Ok(Verified { timestamp })
    } else {
        Err(Rejection::Mismatch)
    }
}

/// Parse `t=<unix>,v1=<hex>[,v1=<hex>...]`.
///
/// Returns the timestamp and **every** `v1=` candidate, decoded. Multiple candidates are
/// how key rotation works: during the overlap window a sender may sign with the new key
/// while the receiver still honours the old one.
fn parse_header(header: &str) -> Result<(i64, Vec<Vec<u8>>), Rejection> {
    let mut timestamp: Option<i64> = None;
    let mut candidates: Vec<Vec<u8>> = Vec::new();

    for part in header.split(',') {
        let part = part.trim();

        if let Some(raw) = part.strip_prefix("t=") {
            if timestamp.is_some() {
                return Err(Rejection::Malformed);
            }

            timestamp = Some(raw.parse::<i64>().map_err(|_| Rejection::Malformed)?);

            continue;
        }

        if let Some(raw) = part.strip_prefix("v1=") {
            // Odd-length or non-hex is malformed, not a mismatch: the sender built the
            // header wrong, which is a different fix than "wrong secret".
            let decoded = hex::decode(raw).map_err(|_| Rejection::Malformed)?;

            if decoded.is_empty() {
                return Err(Rejection::Malformed);
            }

            candidates.push(decoded);

            continue;
        }

        return Err(Rejection::Malformed);
    }

    let timestamp = timestamp.ok_or(Rejection::Malformed)?;

    if candidates.is_empty() {
        return Err(Rejection::Malformed);
    }

    Ok((timestamp, candidates))
}

/// Convenience wrapper: current time from the system clock.
pub fn verify_now(secret: &[u8], header: Option<&str>, body: &[u8]) -> Result<Verified, Rejection> {
    verify(
        secret,
        header,
        body,
        Utc::now().timestamp(),
        DEFAULT_MAX_SKEW_SECS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independently computed: `hmac_sha256("test-secret", "1700000000." + "{\"a\":1}")`.
    /// Hard-coded on purpose — a test that recomputes the MAC with the same library it is
    /// testing cannot catch a construction change (prefix, separator, encoding).
    const GOLDEN_HEADER: &str =
        "t=1700000000,v1=8cb2c3355fca388e9ac2caec004f4d5d7045d74937ab5faad61dc11682247a9f";
    const GOLDEN_BODY: &[u8] = b"{\"a\":1}";
    const SECRET: &[u8] = b"test-secret";
    const NOW: i64 = 1_700_000_010;

    #[test]
    fn the_golden_vector_verifies() {
        let verified = verify(SECRET, Some(GOLDEN_HEADER), GOLDEN_BODY, NOW, 300).expect("valid");

        assert_eq!(verified.timestamp, 1_700_000_000);
    }

    #[test]
    fn a_tampered_body_is_a_mismatch() {
        assert_eq!(
            verify(SECRET, Some(GOLDEN_HEADER), b"{\"a\":2}", NOW, 300),
            Err(Rejection::Mismatch)
        );
    }

    #[test]
    fn rotation_accepts_any_candidate() {
        // The old candidate is bogus, the second is the real one: key rotation in flight.
        let header = format!(
            "t=1700000000,v1={},v1=8cb2c3355fca388e9ac2caec004f4d5d7045d74937ab5faad61dc11682247a9f",
            "00".repeat(32)
        );
        let verified =
            verify(SECRET, Some(&header), GOLDEN_BODY, NOW, 300).expect("second candidate");

        assert_eq!(verified.timestamp, 1_700_000_000);
    }

    #[test]
    fn picks_the_last_candidate_only_when_it_is_the_one_that_matches() {
        // Inverse of the previous test: the *first* candidate matches, the second is junk.
        // A verifier that kept only the last candidate would reject this.
        let header = format!("{GOLDEN_HEADER},v1={}", "11".repeat(32));
        assert!(verify(SECRET, Some(&header), GOLDEN_BODY, NOW, 300).is_ok());
    }

    #[test]
    fn a_missing_header_is_missing() {
        assert_eq!(
            verify(SECRET, None, GOLDEN_BODY, NOW, 300),
            Err(Rejection::Missing)
        );
    }

    #[test]
    fn a_stale_timestamp_is_stale_not_a_mismatch() {
        assert_eq!(
            verify(SECRET, Some(GOLDEN_HEADER), GOLDEN_BODY, NOW + 3600, 300),
            Err(Rejection::Stale)
        );
        // …and the same for the other direction (a clock running ahead).
        assert_eq!(
            verify(SECRET, Some(GOLDEN_HEADER), GOLDEN_BODY, NOW - 3600, 300),
            Err(Rejection::Stale)
        );
    }

    #[test]
    fn the_window_edges_are_inclusive() {
        for now in (1_700_000_000 - 300)..=(1_700_000_000 + 300) {
            assert!(
                verify(SECRET, Some(GOLDEN_HEADER), GOLDEN_BODY, now, 300).is_ok(),
                "now={now}"
            );
        }

        assert_eq!(
            verify(SECRET, Some(GOLDEN_HEADER), GOLDEN_BODY, 1_700_000_301, 300),
            Err(Rejection::Stale)
        );
    }

    #[test]
    fn malformed_headers_are_rejected_as_malformed() {
        let cases = [
            "",
            "t=1700000000",
            "v1=8cb2c335",
            "t=notanumber,v1=8cb2c3355fca388e9ac2caec004f4d5d7045d74937ab5faad61dc11682247a9f",
            "t=1700000000,v1=zz",
            "t=1700000000,v1=",
            "t=1700000000,v1=8cb2c335,extra=1",
            "t=1,t=2,v1=8cb2c3355fca388e9ac2caec004f4d5d7045d74937ab5faad61dc11682247a9f",
        ];

        for header in cases {
            assert_eq!(
                verify(SECRET, Some(header), GOLDEN_BODY, NOW, 300),
                Err(Rejection::Malformed),
                "header={header:?}"
            );
        }
    }

    #[test]
    fn the_wire_codes_are_stable() {
        assert_eq!(Rejection::Missing.wire_code(), "signature_missing");
        assert_eq!(Rejection::Malformed.wire_code(), "signature_malformed");
        assert_eq!(Rejection::Stale.wire_code(), "signature_expired");
        assert_eq!(Rejection::Mismatch.wire_code(), "signature_mismatch");
        assert_eq!(Rejection::Oversized.wire_code(), "body_too_large");
    }

    #[test]
    fn an_empty_secret_fails_closed_even_for_a_correctly_signed_request() {
        // The old version of this test signed with `test-secret` and verified with an
        // empty secret — it failed for the wrong reason and proved nothing. This one
        // signs **with the empty key**, which is exactly what an attacker would do
        // because HMAC accepts a zero-length key.
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(b"").expect("hmac accepts any key length");
        mac.update(b"1700000000.");
        mac.update(GOLDEN_BODY);
        let forged = hex::encode(mac.finalize().into_bytes());
        let header = format!("t=1700000000,v1={forged}");

        assert_eq!(
            verify(b"", Some(&header), GOLDEN_BODY, NOW, 300),
            Err(Rejection::Mismatch),
            "an empty secret must never authenticate anything"
        );
    }

    #[test]
    fn the_window_is_checked_before_the_mac() {
        // A stale **and** wrongly-signed request must report Stale: telling the sender
        // "your MAC is wrong" both hides the attack and tells them they were late.
        let bogus = format!("t=1700000000,v1={}", "00".repeat(32));

        assert_eq!(
            verify(SECRET, Some(&bogus), GOLDEN_BODY, NOW + 9_000, 300),
            Err(Rejection::Stale)
        );
    }

    #[test]
    fn an_oversized_body_is_rejected_before_the_mac() {
        let huge = vec![b'x'; 2048];

        assert_eq!(
            verify_capped(SECRET, Some(GOLDEN_HEADER), &huge, NOW, 300, 1024),
            Err(Rejection::Oversized)
        );

        // …and the default cap is the documented 64 KiB.
        let at_cap = vec![b'x'; DEFAULT_MAX_BODY_BYTES];

        assert_ne!(
            verify(SECRET, Some(GOLDEN_HEADER), &at_cap, NOW, 300),
            Err(Rejection::Oversized),
            "a body exactly at the cap is not oversized"
        );
    }

    #[test]
    fn a_truncated_digest_is_a_mismatch_not_a_match() {
        let short = format!("t=1700000000,v1={}", "8c".repeat(16));

        assert_eq!(
            verify(SECRET, Some(&short), GOLDEN_BODY, NOW, 300),
            Err(Rejection::Mismatch)
        );
    }
}
