use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

const TOOL_HUBRIS_REPORT: &str = "import { report } from 'hubris'";
const TOOL_HUBRIS_REPORT_HUMAN: &str = "import { report_human } from 'hubris'";
const TOOL_OREXIS_REPORT_HUMAN: &str = "import { report_human } from 'orexis'";
const TOOL_OREXIS_ASK_HUMAN: &str = "import { ask_human } from 'orexis'";

pub fn is_wtv_or_wtvj(tool_name: &str) -> bool {
    tool_name == "write_to_var" || tool_name == "write_to_var_json"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolBlockState {
    Pending,
    Running,
    Done,
    Failed,
    HistoryLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCloseLabel {
    Pending,
    Running,
    ReportOnly,
    AskOnly,
    Executed,
    Error,
}

impl fmt::Display for ToolCloseLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolCloseLabel::Pending => write!(f, "Waiting for params..."),
            ToolCloseLabel::Running => write!(f, "Running..."),
            ToolCloseLabel::ReportOnly => write!(f, "Report only"),
            ToolCloseLabel::AskOnly => write!(f, "Waiting for reply"),
            ToolCloseLabel::Executed => write!(f, "Execution complete"),
            ToolCloseLabel::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolBlockData {
    pub tool_name: String,
    pub call_id: Uuid,
    pub agent_type: String,
    pub call_text: String,
    pub result_text: String,
    pub success: bool,
    pub duration_ms: Option<u64>,
    pub state: ToolBlockState,
    pub separate_call_content: Vec<(String, String)>,
}

impl ToolBlockData {
    pub fn extract_wtv_content(call_text: &str) -> Vec<(String, String)> {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(call_text)
            && let Some(content) = v.get("content").and_then(|c| c.as_str())
        {
            return vec![("content".to_string(), content.to_string())];
        }
        Vec::new()
    }

    fn is_report_invocation(&self) -> bool {
        let tool = self.tool_name.as_str();
        if tool == "create_todo" || tool == "compliance_report" {
            return true;
        }
        if tool != "exec" {
            return false;
        }
        let text = &self.call_text;
        text.contains(TOOL_HUBRIS_REPORT)
            || text.contains(TOOL_OREXIS_REPORT_HUMAN)
            || text.contains(TOOL_HUBRIS_REPORT_HUMAN)
    }

    fn is_ask_invocation(&self) -> bool {
        let tool = self.tool_name.as_str();
        if tool.is_empty() {
            return true;
        }
        if tool != "exec" {
            return false;
        }
        self.call_text.contains(TOOL_OREXIS_ASK_HUMAN)
    }

    pub fn close_label(&self) -> ToolCloseLabel {
        match self.state {
            ToolBlockState::Pending => ToolCloseLabel::Pending,
            ToolBlockState::Running => ToolCloseLabel::Running,
            ToolBlockState::Done => {
                if self.is_report_invocation() {
                    ToolCloseLabel::ReportOnly
                } else if self.is_ask_invocation() {
                    ToolCloseLabel::AskOnly
                } else {
                    ToolCloseLabel::Executed
                }
            }
            ToolBlockState::Failed => ToolCloseLabel::Error,
            ToolBlockState::HistoryLost => ToolCloseLabel::Error,
        }
    }
}
