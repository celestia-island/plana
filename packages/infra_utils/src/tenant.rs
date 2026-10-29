//! Tenant resolution primitive — the cross-service `claims.tenant_id`
//! convention (chest's `tenancy.rs` judgment order, arona's auth mint)
//! upstreamed so every service answers identically.
//!
//! Judgment order for one request: a parseable tenant claim wins, then the
//! user record's tenant, then the [`DEFAULT_TENANT_ID`] sentinel — the
//! single-tenant fallback every pre-tenancy row is backfilled to. Zero
//! web-framework dependencies: the axum extractor stays with the consumer,
//! which binds it to that service's own claims type.

use uuid::Uuid;

/// The single-tenant fallback sentinel: `00000000-0000-0000-0000-000000000001`.
///
/// Deliberately not the nil UUID, so "the caller never supplied a tenant"
/// (`Uuid::nil()` shapes) can never masquerade as a resolved default.
pub const DEFAULT_TENANT_ID: Uuid = Uuid::from_u128(1);

/// Resolve the effective tenant for one request.
///
/// - a **parseable** `claims_tenant` wins (unparseable claim text is
///   skipped, not rejected — the token mint is trusted to write UUIDs, and
///   a legacy claim row must not take the whole request down);
/// - otherwise `user_tenant`;
/// - otherwise [`DEFAULT_TENANT_ID`].
pub fn resolve(claims_tenant: Option<&str>, user_tenant: Option<Uuid>) -> Uuid {
    if let Some(t) = claims_tenant.and_then(|s| Uuid::parse_str(s).ok()) {
        return t;
    }
    user_tenant.unwrap_or(DEFAULT_TENANT_ID)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAIM_TENANT: &str = "00000000-0000-0000-0000-0000000000aa";
    const USER_TENANT: &str = "00000000-0000-0000-0000-0000000000bb";

    #[test]
    fn default_is_the_first_tenant_sentinel_not_nil() {
        assert_eq!(DEFAULT_TENANT_ID, Uuid::from_u128(1));
        assert_ne!(DEFAULT_TENANT_ID, Uuid::nil());
    }

    #[test]
    fn parseable_claim_wins_over_user_and_default() {
        assert_eq!(
            resolve(
                Some(CLAIM_TENANT),
                Some(Uuid::parse_str(USER_TENANT).unwrap())
            ),
            Uuid::parse_str(CLAIM_TENANT).unwrap()
        );
        assert_eq!(
            resolve(Some(CLAIM_TENANT), None),
            Uuid::parse_str(CLAIM_TENANT).unwrap()
        );
    }

    #[test]
    fn unparseable_claim_falls_through_to_user() {
        assert_eq!(
            resolve(
                Some("not-a-uuid"),
                Some(Uuid::parse_str(USER_TENANT).unwrap())
            ),
            Uuid::parse_str(USER_TENANT).unwrap()
        );
    }

    #[test]
    fn nothing_supplied_answers_the_default() {
        assert_eq!(resolve(None, None), DEFAULT_TENANT_ID);
        assert_eq!(resolve(Some("not-a-uuid"), None), DEFAULT_TENANT_ID);
        assert_eq!(resolve(Some(""), None), DEFAULT_TENANT_ID);
    }
}
