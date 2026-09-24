//! The lightweight agent handle that code passes around inside a process.
//!
//! Kept separate from `agent::AgentInfo` on purpose: `AgentContext` holds only
//! what a component needs in order to know whose work it is doing, and stays
//! out of the wire format.

use crate::agent::Agent;
use plana_core::AgentBadge;

/// Minimal handle to the agent a piece of work belongs to: its kind, its
/// instance id, and the badge when one was assigned. `Clone` but not
/// serializable, so it is passed between components instead of being sent on a
/// wire.
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// Kind of the agent holding this context.
    pub agent_type: Agent,
    /// Instance id as a plain string, where `AgentInfo::agent_id` uses the `AgentId`
    /// newtype.
    pub agent_id: String,
    /// Badge of this instance, when one has been assigned.
    pub agent_number: Option<AgentBadge>,
}
