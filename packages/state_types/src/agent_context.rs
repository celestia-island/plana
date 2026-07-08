use crate::agent::Agent;
use arona_core::AgentBadge;

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub agent_type: Agent,
    pub agent_id: String,
    pub agent_number: Option<AgentBadge>,
}
