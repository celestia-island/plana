//! `Conversation.*` DTOs: consultations between two agents about one file.
//!
//! A thread is keyed by `conversation_id` and moves ask to reply, or escalates
//! to a human and is later resolved. The bridge parses all four actions, so
//! these frames survive a JSON-RPC round trip.

use serde::{Deserialize, Serialize};

/// One `Conversation`-namespace action, serde-tagged by `action`.
///
/// Wire shape:
/// `{"type": "Conversation", "data": {"action": "AskAgent", ...}}`. Every
/// field is a plain string — agent ids, a file path, prose — so both peers
/// have to agree out of band on how the `what`/`why`/`how` values are formed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ConversationMessage {
    /// `Conversation.AskAgent` — opens or continues the thread keyed by
    /// `conversation_id`: `from_agent` asks `to_agent` about `file_path`, stating
    /// what it needs to know and the why/how behind that need.
    AskAgent {
        conversation_id: String,
        from_agent: String,
        to_agent: String,
        file_path: String,
        reasoning_what: String,
        reasoning_why: String,
        reasoning_how: String,
    },
    /// `Conversation.ReplyAgent` — the consulted agent answers on the same
    /// `conversation_id` with the answer prose and `message_type`, a free-form
    /// tag describing the kind of answer (a string, not an enum).
    ReplyAgent {
        conversation_id: String,
        from_agent: String,
        to_agent: String,
        answer_what: String,
        answer_why: String,
        answer_how: String,
        message_type: String,
    },
    /// `Conversation.Escalated` — the thread could not be settled between the two
    /// agents and is handed to a human: `human_consultation_id` is the
    /// consultation the human flow tracks and `summary` is the prose handed over.
    Escalated {
        conversation_id: String,
        human_consultation_id: String,
        summary: String,
    },
    /// `Conversation.Resolved` — closes the thread. `resolution` is prose, so a
    /// consumer that needs a machine-readable outcome must not look for one here.
    Resolved {
        conversation_id: String,
        resolution: String,
    },
}
