use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ConversationMessage {
    AskAgent {
        conversation_id: String,
        from_agent: String,
        to_agent: String,
        file_path: String,
        reasoning_what: String,
        reasoning_why: String,
        reasoning_how: String,
    },
    ReplyAgent {
        conversation_id: String,
        from_agent: String,
        to_agent: String,
        answer_what: String,
        answer_why: String,
        answer_how: String,
        message_type: String,
    },
    Escalated {
        conversation_id: String,
        human_consultation_id: String,
        summary: String,
    },
    Resolved {
        conversation_id: String,
        resolution: String,
    },
}
