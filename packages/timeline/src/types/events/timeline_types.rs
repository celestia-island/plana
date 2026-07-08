use serde::{Deserialize, Serialize};
use std::fmt;

use super::mcp_block::McpBlockData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupState {
    Mutable,
    Finalized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillBlockStatus {
    Thinking,
    Executing,
    Done,
    Failed,
    Retried,
}

impl fmt::Display for SkillBlockStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillBlockStatus::Thinking => write!(f, "Thinking"),
            SkillBlockStatus::Executing => write!(f, "Executing"),
            SkillBlockStatus::Done => write!(f, "Done thinking"),
            SkillBlockStatus::Failed => write!(f, "Execution failed"),
            SkillBlockStatus::Retried => write!(f, "Retrying"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimelineContentKind {
    Text,
    Thinking,
    DeepThinking,
    Error,
}

#[derive(Debug, Clone)]
pub struct TimelineContentBlock {
    pub text: String,
    pub kind: TimelineContentKind,
}

#[derive(Debug, Clone)]
pub enum TimelineSegmentBlock {
    Content(TimelineContentBlock),
    Mcp(McpBlockData),
}

#[derive(Debug, Clone)]
pub struct TimelineHumanGroup {
    pub content: String,
    pub timestamp: String,
    pub agent_number: Option<String>,
    pub username: String,
    pub status_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TimelineAskHumanGroup {
    pub agent_number: String,
    pub agent_type: String,
    pub question: String,
    pub options: Vec<String>,
    pub recommended: Option<String>,
    pub answer: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum TimelineGroup {
    Human(TimelineHumanGroup),
    Skill(Box<super::segment_processing::TimelineGroupData>),
    AskHuman(TimelineAskHumanGroup),
}

impl TimelineGroup {
    pub fn state(&self) -> GroupState {
        match self {
            TimelineGroup::Human(_) | TimelineGroup::AskHuman(_) => GroupState::Finalized,
            TimelineGroup::Skill(data) => data.state,
        }
    }
}
