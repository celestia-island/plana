use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

const MCP_HUBRIS_REPORT: &str = "import { report } from 'hubris'";
const MCP_HUBRIS_REPORT_HUMAN: &str = "import { report_human } from 'hubris'";
const MCP_OREXIS_REPORT_HUMAN: &str = "import { report_human } from 'orexis'";
const MCP_OREXIS_ASK_HUMAN: &str = "import { ask_human } from 'orexis'";

pub fn is_wtv_or_wtvj(tool_name: &str) -> bool {
    tool_name == "write_to_var" || tool_name == "write_to_var_json"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpBlockState {
    Pending,
    Running,
    Done,
    Failed,
    HistoryLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpCloseLabel {
    Pending,
    Running,
    ReportOnly,
    AskOnly,
    Executed,
    Error,
}

impl fmt::Display for McpCloseLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpCloseLabel::Pending => write!(f, "Waiting for params..."),
            McpCloseLabel::Running => write!(f, "Running..."),
            McpCloseLabel::ReportOnly => write!(f, "Report only"),
            McpCloseLabel::AskOnly => write!(f, "Waiting for reply"),
            McpCloseLabel::Executed => write!(f, "Execution complete"),
            McpCloseLabel::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct McpBlockData {
    pub tool_name: String,
    pub call_id: Uuid,
    pub agent_type: String,
    pub call_text: String,
    pub result_text: String,
    pub success: bool,
    pub duration_ms: Option<u64>,
    pub state: McpBlockState,
    pub separate_call_content: Vec<(String, String)>,
}

impl McpBlockData {
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
        text.contains(MCP_HUBRIS_REPORT)
            || text.contains(MCP_OREXIS_REPORT_HUMAN)
            || text.contains(MCP_HUBRIS_REPORT_HUMAN)
    }

    fn is_ask_invocation(&self) -> bool {
        let tool = self.tool_name.as_str();
        if tool.is_empty() {
            return true;
        }
        if tool != "exec" {
            return false;
        }
        self.call_text.contains(MCP_OREXIS_ASK_HUMAN)
    }

    pub fn close_label(&self) -> McpCloseLabel {
        match self.state {
            McpBlockState::Pending => McpCloseLabel::Pending,
            McpBlockState::Running => McpCloseLabel::Running,
            McpBlockState::Done => {
                if self.is_report_invocation() {
                    McpCloseLabel::ReportOnly
                } else if self.is_ask_invocation() {
                    McpCloseLabel::AskOnly
                } else {
                    McpCloseLabel::Executed
                }
            }
            McpBlockState::Failed => McpCloseLabel::Error,
            McpBlockState::HistoryLost => McpCloseLabel::Error,
        }
    }
}
