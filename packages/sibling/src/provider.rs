//! Dynamic endpoint source callback.
//!
//! The facility must not depend on any consumer's schema (chest's
//! `backend_connections` is a chest table), so DB-backed endpoint sources are
//! expressed as this callback instead: the consumer owns the table read and
//! returns the ordered endpoint list.

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
