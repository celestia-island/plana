use crate::enums::{
    ConversationMessageType, ConversationStatus, FileOperationType, ObservationType,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct LlmProviderCallResult {
    pub model: String,
    pub tokens: String,
    pub response: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, PartialEq, Eq, Hash)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct AgentReference {
    pub agent_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_badge: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct FileLineRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ConflictInfo {
    pub conflict_id: String,
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_range: Option<FileLineRange>,
    pub conflicting_agent: AgentReference,
    pub operation_type: FileOperationType,
    pub since: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ObserverInfo {
    pub agent: AgentReference,
    pub observation_type: ObservationType,
    pub since: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct NotifyFileOperationResult {
    pub file_path: String,
    pub observers_count: usize,
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ListFileObserversResult {
    pub file_path: String,
    pub observers: Vec<ObserverInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct AgentReasoning {
    pub what: String,
    pub why: String,
    pub how: String,
}

/// Structured reason-evaluation triples for agent self-reflection.
/// Semantically distinct from [`AgentReasoning`] — this is the context of
/// the ongoing conversation, not the agent's own reasoning chain.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ConversationContext {
    pub what: String,
    pub why: String,
    pub how: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct FileAnchor {
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_hash: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ConversationMessage {
    pub id: String,
    pub author: AgentReference,
    pub content: AgentReasoning,
    pub message_type: ConversationMessageType,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct AgentConversation {
    pub id: String,
    pub topic: String,
    pub status: ConversationStatus,
    pub initiator: AgentReference,
    pub participants: Vec<AgentReference>,
    pub context: ConversationContext,
    pub messages: Vec<ConversationMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_anchor: Option<FileAnchor>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_consultation_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct AskAgentResult {
    pub conversation_id: String,
    pub status: ConversationStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ReplyAgentResult {
    pub conversation_id: String,
    pub status: ConversationStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct EscalateConversationResult {
    pub conversation_id: String,
    pub human_consultation_id: String,
    pub escalated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ListConversationsResult {
    pub count: usize,
    pub conversations: Vec<AgentConversation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct GetConversationResult {
    pub conversation: AgentConversation,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct LlmProviderCallParams {
    pub tier: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/haplotes.ts")]
pub struct SubscribeTriggerParams {
    pub topic_pattern: String,
    pub agent_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_agent_reference_with_badge() -> Result<()> {
        let ar = AgentReference {
            agent_type: "WebAutomation".to_string(),
            instance_badge: Some("#002".to_string()),
        };
        let json = serde_json::to_string(&ar)?;
        let de: AgentReference = serde_json::from_str(&json)?;
        assert_eq!(de.agent_type, "WebAutomation");
        assert_eq!(de.instance_badge, Some("#002".to_string()));
        Ok(())
    }

    #[test]
    fn test_agent_reference_without_badge() -> Result<()> {
        let ar = AgentReference {
            agent_type: "HubRis".to_string(),
            instance_badge: None,
        };
        let json = serde_json::to_string(&ar)?;
        let de: AgentReference = serde_json::from_str(&json)?;
        assert_eq!(de.instance_badge, None);
        Ok(())
    }

    #[test]
    fn test_conflict_info_serialization() -> Result<()> {
        let ci = ConflictInfo {
            conflict_id: "cflict-001".to_string(),
            file_path: "src/auth.rs".to_string(),
            line_range: Some(FileLineRange { start: 10, end: 20 }),
            conflicting_agent: AgentReference {
                agent_type: "WebAutomation".to_string(),
                instance_badge: Some("#002".to_string()),
            },
            operation_type: FileOperationType::Editing,
            since: "2026-05-11T10:30:00Z".to_string(),
        };
        let json = serde_json::to_string(&ci)?;
        let de: ConflictInfo = serde_json::from_str(&json)?;
        assert_eq!(de.conflict_id, "cflict-001");
        assert_eq!(de.file_path, "src/auth.rs");
        assert_eq!(de.operation_type, FileOperationType::Editing);
        Ok(())
    }

    #[test]
    fn test_agent_reasoning_structure() -> Result<()> {
        let r = AgentReasoning {
            what: "refactoring auth".to_string(),
            why: "security audit requirement".to_string(),
            how: "extract AuthProvider trait".to_string(),
        };
        let json = serde_json::to_string(&r)?;
        let de: AgentReasoning = serde_json::from_str(&json)?;
        assert_eq!(de.what, "refactoring auth");
        assert_eq!(de.why, "security audit requirement");
        Ok(())
    }

    #[test]
    fn test_conversation_status_roundtrip() -> Result<()> {
        let statuses = vec![
            ConversationStatus::Active,
            ConversationStatus::Resolved,
            ConversationStatus::Deadlocked,
            ConversationStatus::Escalated,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s)?;
            let de: ConversationStatus = serde_json::from_str(&json)?;
            assert_eq!(s, de);
        }
        Ok(())
    }

    #[test]
    fn test_conversation_message_types() -> Result<()> {
        let types = vec![
            ConversationMessageType::Question,
            ConversationMessageType::Answer,
            ConversationMessageType::Clarification,
            ConversationMessageType::Objection,
            ConversationMessageType::CounterProposal,
            ConversationMessageType::Resolution,
        ];
        assert_eq!(types.len(), 6);
        for t in types {
            let json = serde_json::to_string(&t)?;
            let de: ConversationMessageType = serde_json::from_str(&json)?;
            assert_eq!(t, de);
        }
        Ok(())
    }

    #[test]
    fn test_notify_result_serialization() -> Result<()> {
        let r = NotifyFileOperationResult {
            file_path: "src/main.rs".to_string(),
            observers_count: 2,
            conflicts: vec![],
        };
        let json = serde_json::to_string(&r)?;
        let de: NotifyFileOperationResult = serde_json::from_str(&json)?;
        assert_eq!(de.observers_count, 2);
        assert!(de.conflicts.is_empty());
        Ok(())
    }

    #[test]
    fn test_ask_agent_result_serialization() -> Result<()> {
        let r = AskAgentResult {
            conversation_id: "conv-001".to_string(),
            status: ConversationStatus::Active,
            created_at: "2026-05-11T10:30:00Z".to_string(),
        };
        let json = serde_json::to_string(&r)?;
        let de: AskAgentResult = serde_json::from_str(&json)?;
        assert_eq!(de.conversation_id, "conv-001");
        Ok(())
    }
}
