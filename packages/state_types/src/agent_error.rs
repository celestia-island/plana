pub use plana_core::{AgentErrorCode, StructuredAgentError};

// The context vocabularies are declared ONCE, in celestia-types (the contract
// crate that also owns the ts-rs bindings the webui consumes); this shim
// re-exports them so the engine (which takes its agent-error vocabulary from
// here) writes `reason` / `interrupted` values through the typed enums
// instead of raw string literals.
///
/// Consumer-shaped root-surface pin: a doctest compiles against this crate
/// as an EXTERNAL crate, so the imports resolve only when lib.rs's root
/// `pub use agent_error::…` actually carries the names — the 0.5.1 publish
/// shipped the vocabularies module-only, and only the engine's compile
/// (not any in-crate test, which resolves through `self`) noticed.
///
/// ```rust
/// use plana_state_sync::{AgentErrorReason, AgentErrorCode, AgentInterruptReason, StructuredAgentError};
///
/// let reason = AgentErrorReason::PeerUnreachable;
/// assert_eq!(reason.wire_form(), "peer_unreachable");
/// assert_eq!(AgentInterruptReason::Restart.wire_form(), "restart");
/// let code: AgentErrorCode = serde_json::from_str("\"chain_failed\"").unwrap();
/// assert_eq!(code, AgentErrorCode::ChainFailed);
/// let error = StructuredAgentError::new(AgentErrorCode::AgentBusy);
/// assert_eq!(error.code, AgentErrorCode::AgentBusy);
/// ```
pub use plana_celestia_types::{AgentErrorReason, AgentInterruptReason};

#[cfg(test)]
mod tests {
    //! Lockstep proof for the two serialization twins of `AgentErrorCode`:
    //! `plana_core::AgentErrorCode` (this shim re-exports it) and
    //! `plana_celestia_types::AgentErrorCode` (the published domain-profile
    //! copy) must carry the same variants, in the same order, with the same
    //! snake_case wire values. The two defining crates deliberately do not
    //! depend on each other, so the only crate that sees both copies — this
    //! one — owns the check.
    use std::collections::BTreeSet;

    /// Wire value of a core-twin variant (the real serde contract, not a
    /// hand-copied literal).
    fn core_wire(code: &plana_core::AgentErrorCode) -> String {
        serde_json::to_string(code).unwrap()
    }

    /// Wire value of a celestia-twin variant.
    fn celestia_wire(code: &plana_celestia_types::AgentErrorCode) -> String {
        serde_json::to_string(code).unwrap()
    }

    /// Compile-time exhaustiveness probes: no wildcard arm in either match,
    /// so a variant added to either twin fails to compile here until it is
    /// catalogued. When that happens, add the variant to the probe AND to
    /// the corresponding census list below, in declaration order.
    fn probe_core(code: &plana_core::AgentErrorCode) {
        use plana_core::AgentErrorCode as C;
        match code {
            C::ModelNoProviders
            | C::ModelNoModels
            | C::ModelTierMismatch
            | C::ModelAllExcluded
            | C::ModelEnvIncomplete
            | C::ModelSelectionRetryExhausted
            | C::ModelNoCapableModel
            | C::LlmCallFailed
            | C::LlmEmptyResponse
            | C::LlmRateLimited
            | C::LlmAuthFailed
            | C::LlmTimeout
            | C::CosmosNoConnection
            | C::CosmosToolFailed
            | C::CosmosLocalUnavailable
            | C::ChainMaxDepth
            | C::ChainCycle
            | C::ChainFailed
            | C::SkillFailed
            | C::SkillEmptyOutput
            | C::SkillMissingReport
            | C::DependencyDown
            | C::AgentBusy
            | C::BudgetExhausted => {}
        }
    }

    fn probe_celestia(code: &plana_celestia_types::AgentErrorCode) {
        use plana_celestia_types::AgentErrorCode as T;
        match code {
            T::ModelNoProviders
            | T::ModelNoModels
            | T::ModelTierMismatch
            | T::ModelAllExcluded
            | T::ModelEnvIncomplete
            | T::ModelSelectionRetryExhausted
            | T::ModelNoCapableModel
            | T::LlmCallFailed
            | T::LlmEmptyResponse
            | T::LlmRateLimited
            | T::LlmAuthFailed
            | T::LlmTimeout
            | T::CosmosNoConnection
            | T::CosmosToolFailed
            | T::CosmosLocalUnavailable
            | T::ChainMaxDepth
            | T::ChainCycle
            | T::ChainFailed
            | T::SkillFailed
            | T::SkillEmptyOutput
            | T::SkillMissingReport
            | T::DependencyDown
            | T::AgentBusy
            | T::BudgetExhausted => {}
        }
    }

    /// Both twins must serialize to the same wire values in the same order.
    ///
    /// Catches, per twin: a variant missing from the census (length or
    /// element mismatch), order drift (element-wise comparison), and wire
    /// drift (values come from serde, not literals). The exhaustive probes
    /// above force any new variant to land in this file; keep the census
    /// lists in the same declaration order as their enums.
    #[test]
    fn agent_error_code_twins_stay_in_lockstep() {
        use plana_celestia_types::AgentErrorCode as T;
        use plana_core::AgentErrorCode as C;

        let core = [
            C::ModelNoProviders,
            C::ModelNoModels,
            C::ModelTierMismatch,
            C::ModelAllExcluded,
            C::ModelEnvIncomplete,
            C::ModelSelectionRetryExhausted,
            C::ModelNoCapableModel,
            C::LlmCallFailed,
            C::LlmEmptyResponse,
            C::LlmRateLimited,
            C::LlmAuthFailed,
            C::LlmTimeout,
            C::CosmosNoConnection,
            C::CosmosToolFailed,
            C::CosmosLocalUnavailable,
            C::ChainMaxDepth,
            C::ChainCycle,
            C::ChainFailed,
            C::SkillFailed,
            C::SkillEmptyOutput,
            C::SkillMissingReport,
            C::DependencyDown,
            C::AgentBusy,
            C::BudgetExhausted,
        ];
        let celestia = [
            T::ModelNoProviders,
            T::ModelNoModels,
            T::ModelTierMismatch,
            T::ModelAllExcluded,
            T::ModelEnvIncomplete,
            T::ModelSelectionRetryExhausted,
            T::ModelNoCapableModel,
            T::LlmCallFailed,
            T::LlmEmptyResponse,
            T::LlmRateLimited,
            T::LlmAuthFailed,
            T::LlmTimeout,
            T::CosmosNoConnection,
            T::CosmosToolFailed,
            T::CosmosLocalUnavailable,
            T::ChainMaxDepth,
            T::ChainCycle,
            T::ChainFailed,
            T::SkillFailed,
            T::SkillEmptyOutput,
            T::SkillMissingReport,
            T::DependencyDown,
            T::AgentBusy,
            T::BudgetExhausted,
        ];

        // The census covers every variant of its twin (compile-enforced via
        // the probes) and lists none of them twice.
        for code in &core {
            probe_core(code);
        }
        for code in &celestia {
            probe_celestia(code);
        }
        let distinct: BTreeSet<_> = core.iter().map(core_wire).collect();
        assert_eq!(distinct.len(), core.len(), "duplicate core wire value");
        let distinct: BTreeSet<_> = celestia.iter().map(celestia_wire).collect();
        assert_eq!(
            distinct.len(),
            celestia.len(),
            "duplicate celestia wire value"
        );

        // Same length, same order, same wire value — the lockstep contract.
        assert_eq!(core.len(), celestia.len(), "twin variant counts differ");
        for (i, (c, t)) in core.iter().zip(celestia.iter()).enumerate() {
            assert_eq!(
                core_wire(c),
                celestia_wire(t),
                "twin drift at declaration position {i}: core {c:?} vs celestia {t:?}"
            );
        }
    }
}
