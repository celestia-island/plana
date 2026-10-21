//! Local-suite cloud registration state — the shared wire contract.
//!
//! A LOCAL member of the celestia suite (evernight node, arona gateway,
//! entelecheia/scepter instance, self-hosted chest) exposes this block as
//! a service-owned extension field (`"registration"`) of its `/api/health`
//! response. The desktop client's registration wizard reads exactly this
//! block to answer "is this local instance bound to a cloud account yet,
//! and if so, whose" — the detection precondition of the 2026-10-21
//! local-suite registration project.
//!
//! Placement rules (deliberate):
//! - This type lives in the plana foundation because ≥2 services consume
//!   the same shape ([[layers]]: reusable capability goes upstream first).
//! - It is NOT part of [`crate::protocol_core::HealthResponse`]: that base
//!   is flattened into every backend's health payload family-wide, and
//!   extending it would change the contract of consumers that will never
//!   register (cloud services, worker probes). Each registering service
//!   embeds `registration` in its own health extension fields, the same
//!   way scepter already carries `needs_setup` beside the flattened base.
//!
//! Field semantics:
//! - `instance_id` is a stable per-installation identifier generated once
//!   by the owning service and persisted as a class-A bootstrap root (a
//!   0600 env/config file — never a database row). It MAY be derived from
//!   [`crate::protocol_core::identity::machine_fingerprint()`] or reuse a
//!   service-native identity (evernight's registry `node_id`); consumers
//!   must treat it as opaque.
//! - `owner_account` / `owner_group` are opaque references into the cloud
//!   account system's id space (the chest subject and the group the
//!   resource auto-mounted to — a team account attaches the instance to
//!   the user's group, not merely the user; 2026-10-21 user ruling).
//!   The registration authority is the cloud side; the local config file
//!   is the binding's authoritative carrier on the node.
//! - `registry_endpoint` names the cloud registration surface the
//!   instance is bound to (e.g. an enrollment gateway base URL), so a
//!   re-registration flow can tell WHICH cloud it belongs to.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The registration block of a local suite member's `/api/health`
/// extension fields. Unregistered instances carry only `instance_id` and
/// `registered: false`; every other field is omitted while unset so the
/// common case stays a two-key object.
///
/// Invariant: `registered == true` implies `owner_account.is_some()` (a
/// binding always has an owning account). The [`RegistrationState::registered`]
/// constructor enforces it; direct struct construction bypasss it and takes
/// on the obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "registrationTypes.ts")]
pub struct RegistrationState {
    /// Stable per-installation identifier (opaque; see module docs).
    pub instance_id: String,
    /// False until the instance has been bound to a cloud account/group.
    pub registered: bool,
    /// Opaque cloud account reference (the minting/owning account).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub owner_account: Option<String>,
    /// Opaque group reference (set when a team account auto-mounted the
    /// instance as a group resource; absent for personal ownership).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub owner_group: Option<String>,
    /// The cloud registration surface the instance is bound to.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub registry_endpoint: Option<String>,
    /// RFC 3339 UTC timestamp of the binding.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub registered_at: Option<String>,
}

impl RegistrationState {
    /// The shape an unregistered instance reports: identity known, no
    /// binding. Serializes to exactly two keys.
    pub fn unregistered(instance_id: impl Into<String>) -> Self {
        Self {
            instance_id: instance_id.into(),
            registered: false,
            owner_account: None,
            owner_group: None,
            registry_endpoint: None,
            registered_at: None,
        }
    }

    /// The shape after a successful binding. `owner_account` is required
    /// (a binding always has an owning account); the group, endpoint and
    /// timestamp enrich it.
    pub fn registered(
        instance_id: impl Into<String>,
        owner_account: impl Into<String>,
        owner_group: Option<String>,
        registry_endpoint: Option<String>,
        registered_at: Option<String>,
    ) -> Self {
        Self {
            instance_id: instance_id.into(),
            registered: true,
            owner_account: Some(owner_account.into()),
            owner_group,
            registry_endpoint,
            registered_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregistered_serializes_to_exactly_two_keys() {
        let state = RegistrationState::unregistered("scepter-example");
        let json = serde_json::to_value(&state).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "instance_id": "scepter-example",
                "registered": false,
            })
        );
    }

    #[test]
    fn full_state_round_trips() {
        let state = RegistrationState::registered(
            "arona-example",
            "acct-123",
            Some("group-42".to_string()),
            Some("https://gateway.celestia.world".to_string()),
            Some("2026-10-21T02:00:00Z".to_string()),
        );
        let json = serde_json::to_string(&state).expect("serialize");
        let back: RegistrationState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, state);
        assert!(back.registered);
        assert_eq!(back.owner_account.as_deref(), Some("acct-123"));
        assert_eq!(back.owner_group.as_deref(), Some("group-42"));
    }

    #[test]
    fn unknown_fields_are_ignored_for_forward_compatibility() {
        // A future contract version may add fields; old readers (the
        // wizard ships on its own cadence) must keep parsing, same
        // forward-compat posture as the flattened HealthResponse base.
        let json = serde_json::json!({
            "instance_id": "evernight-example",
            "registered": true,
            "owner_account": "acct-1",
            "some_future_field": { "nested": true },
        });
        let state: RegistrationState =
            serde_json::from_value(json).expect("forward-compatible parse");
        assert!(state.registered);
        assert_eq!(state.owner_account.as_deref(), Some("acct-1"));
        assert_eq!(state.registry_endpoint, None);
    }

    #[test]
    fn explicit_null_is_equivalent_to_absence() {
        // Producers vary: some JSON stacks emit explicit nulls. Both forms
        // must deserialize to None — the wire contract is "key absent",
        // and null is the same statement.
        let json = serde_json::json!({
            "instance_id": "x",
            "registered": false,
            "owner_account": null,
            "owner_group": null,
        });
        let state: RegistrationState = serde_json::from_value(json).expect("null parse");
        assert_eq!(state.owner_account, None);
        assert_eq!(state.owner_group, None);
    }

    #[test]
    fn registered_constructor_invariants_hold() {
        // `registered: true` MUST carry an owner account — a binding always
        // has an owning account (module docs). The constructor enforces it;
        // direct struct construction bypasses it, so this pin plus the doc
        // note define the obligation for direct constructors.
        let state = RegistrationState::registered("i", "acct", None, None, None);
        assert!(state.registered);
        assert!(state.owner_account.is_some());
        // Personal ownership (no group) serializes to exactly three keys —
        // the precise key set the wizard's TS types (`#[ts(optional)]`)
        // promise: every optional key may be absent.
        let json = serde_json::to_value(&state).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "instance_id": "i",
                "registered": true,
                "owner_account": "acct",
            })
        );
    }
}
