//! Dynamic endpoint / credential source callbacks.
//!
//! The facility must not depend on any consumer's schema (chest's
//! `backend_connections` is a chest table), so DB-backed endpoint sources are
//! expressed as this callback instead: the consumer owns the table read and
//! returns the ordered endpoint list.

use secrecy::SecretString;

/// Ordered sibling endpoint provider.
///
/// Queried on **every** call, so a consumer can read its DB (and cache as it
/// sees fit) — endpoint changes take effect immediately instead of requiring
/// a restart. Return an empty `Vec` when no endpoint is currently usable;
/// the call then reports [`crate::SiblingError::AllEndpointsDown`] with an
/// empty/shorter `tried` list. Log read failures inside your implementation;
/// this trait deliberately cannot fail — an endpoint list that is empty is
/// the "no sibling available" signal.
pub trait EndpointProvider: Send + Sync {
    /// Ordered endpoint base URLs (`scheme://host[:port]`), most preferred
    /// first.
    fn endpoints(&self) -> Vec<String>;
}

/// Dynamic bearer credential callback.
///
/// The mirror of [`EndpointProvider`] for credentials: queried on **every**
/// attempt, so a consumer whose token rotates at runtime (DB-encrypted
/// tokens, env-only-at-call-time secrets) stays fresh without a restart.
/// Return `None` to send the attempt without an `Authorization` header.
///
/// Redaction stays structural: return a [`SecretString`], never a bare
/// `String` — the facility renders tokens nowhere (errors, spans, `Debug`).
pub trait BearerProvider: Send + Sync {
    /// The bearer token for this attempt, or `None` for no auth header.
    fn bearer(&self) -> Option<SecretString>;
}
