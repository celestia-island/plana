mod parsing;
mod rendering;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StreamChunkKind {
    #[default]
    Text,
    Thinking,
    DeepThinking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamToolEvent {
    ToolCall {
        tool_name: String,
        call_id: Uuid,
        params_summary: Option<String>,
        agent_type: Option<String>,
    },
    ToolResult {
        tool_name: String,
        call_id: Uuid,
        result: String,
        success: bool,
        duration_ms: Option<u64>,
        agent_type: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamSegment {
    Text {
        text: String,
        #[serde(default)]
        message_id: Option<Uuid>,
    },
    Thinking {
        text: String,
        #[serde(default)]
        message_id: Option<Uuid>,
    },
    DeepThinking {
        text: String,
        #[serde(default)]
        message_id: Option<Uuid>,
    },
    ToolCall {
        tool_name: String,
        call_id: Uuid,
        params: Value,
        #[serde(default)]
        agent_type: Option<String>,
        #[serde(default)]
        message_id: Option<Uuid>,
    },
    ToolResult {
        tool_name: String,
        call_id: Uuid,
        success: bool,
        data: Value,
        #[serde(default)]
        duration_ms: Option<u64>,
        #[serde(default)]
        agent_type: Option<String>,
        #[serde(default)]
        message_id: Option<Uuid>,
    },
}

impl StreamSegment {
    pub fn text(&self) -> &str {
        match self {
            Self::Text { text, .. }
            | Self::Thinking { text, .. }
            | Self::DeepThinking { text, .. } => text,
            Self::ToolCall { params, .. } => match params {
                Value::String(s) => s,
                _ => "",
            },
            Self::ToolResult { data, .. } => match data {
                Value::String(s) => s,
                Value::Null => "",
                _ => "",
            },
        }
    }

    pub fn text_or_json(&self) -> String {
        match self {
            Self::Text { text, .. }
            | Self::Thinking { text, .. }
            | Self::DeepThinking { text, .. } => text.clone(),
            Self::ToolCall { params, .. } | Self::ToolResult { data: params, .. } => match params {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                other => other.to_string(),
            },
        }
    }

    pub fn is_tool(&self) -> bool {
        matches!(self, Self::ToolCall { .. } | Self::ToolResult { .. })
    }

    pub fn is_thinking(&self) -> bool {
        matches!(self, Self::Thinking { .. } | Self::DeepThinking { .. })
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    pub fn chunk_kind(&self) -> Option<StreamChunkKind> {
        match self {
            Self::Text { .. } => Some(StreamChunkKind::Text),
            Self::Thinking { .. } => Some(StreamChunkKind::Thinking),
            Self::DeepThinking { .. } => Some(StreamChunkKind::DeepThinking),
            _ => None,
        }
    }

    pub fn tool_name(&self) -> Option<&str> {
        match self {
            Self::ToolCall { tool_name, .. } => Some(tool_name),
            Self::ToolResult { tool_name, .. } => Some(tool_name),
            _ => None,
        }
    }

    pub fn call_id(&self) -> Option<Uuid> {
        match self {
            Self::ToolCall { call_id, .. } => Some(*call_id),
            Self::ToolResult { call_id, .. } => Some(*call_id),
            _ => None,
        }
    }

    pub fn agent_type(&self) -> Option<&str> {
        match self {
            Self::ToolCall { agent_type, .. } => agent_type.as_deref(),
            Self::ToolResult { agent_type, .. } => agent_type.as_deref(),
            _ => None,
        }
    }

    pub fn tool_params(&self) -> Option<&Value> {
        match self {
            Self::ToolCall { params, .. } => Some(params),
            _ => None,
        }
    }

    pub fn tool_result_data(&self) -> Option<&Value> {
        match self {
            Self::ToolResult { data, .. } => Some(data),
            _ => None,
        }
    }

    pub fn tool_result_success(&self) -> Option<bool> {
        match self {
            Self::ToolResult { success, .. } => Some(*success),
            _ => None,
        }
    }
}

pub use parsing::LlmStreamBuilder;
pub use rendering::LlmStream;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[derive(serde::Serialize)]
    struct PathParam<'a> {
        path: &'a str,
    }

    fn empty() -> serde_json::Value {
        serde_json::Value::Object(Default::default())
    }

    #[test]
    fn test_builder_text_segments() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk("hello ", StreamChunkKind::Text);
        builder.push_chunk("world", StreamChunkKind::Text);
        assert_eq!(builder.as_str(), "hello world");
        assert_eq!(builder.segments().len(), 1);

        let stream = builder.seal_coalesced();
        assert_eq!(stream.segments().len(), 1);
        assert_eq!(stream.raw_text(), "hello world");
        Ok(())
    }

    #[test]
    fn test_builder_mixed_segments() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk("thinking...", StreamChunkKind::Thinking);
        let call_id = Uuid::now_v7();
        builder.push_tool_call(
            "read_file".to_string(),
            call_id,
            Some(serde_json::to_value(PathParam { path: "/test" }).unwrap_or_default()),
            None,
        );
        builder.push_tool_result(
            "read_file".to_string(),
            call_id,
            Value::String("file contents".to_string()),
            true,
            Some(150),
            None,
        );
        builder.push_chunk("Here is the file:", StreamChunkKind::Text);

        assert_eq!(builder.segments().len(), 4);
        assert_eq!(builder.as_str(), "Here is the file:");

        let stream = builder.seal();
        let tools: Vec<_> = stream.tool_segments().collect();
        assert_eq!(tools.len(), 2);
        assert!(matches!(tools[0], StreamSegment::ToolCall { .. }));
        assert!(matches!(tools[1], StreamSegment::ToolResult { .. }));
        assert_eq!(tools[0].text(), "");
        assert_eq!(tools[1].text(), "file contents");
        assert_eq!(tools[0].call_id(), Some(call_id));
        assert_eq!(tools[1].call_id(), Some(call_id));
        Ok(())
    }

    #[test]
    fn test_llm_stream_from_plain_text() -> Result<()> {
        let stream = LlmStream::from_plain_text("test content");
        assert_eq!(stream.raw_text(), "test content");
        assert_eq!(stream.segments().len(), 1);
        Ok(())
    }

    #[test]
    fn test_display_text_vs_thinking() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk("let me think", StreamChunkKind::Thinking);
        builder.push_chunk("Here is my answer", StreamChunkKind::Text);

        let stream = builder.seal();
        assert_eq!(stream.thinking_text(), "let me think");
        assert_eq!(stream.display_text(), "Here is my answer");
        Ok(())
    }

    #[test]
    fn test_every_segment_has_text() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk("report intro", StreamChunkKind::Text);
        let call_id = Uuid::now_v7();
        builder.push_tool_call("report".to_string(), call_id, Some(empty()), None);
        builder.push_tool_result(
            "report".to_string(),
            call_id,
            Value::String("This is the actual report content".to_string()),
            true,
            None,
            None,
        );
        builder.push_chunk("conclusion", StreamChunkKind::Text);

        let stream = builder.seal();
        let full_text = stream.raw_text();
        assert!(full_text.contains("report intro"));
        assert!(full_text.contains("This is the actual report content"));
        assert!(full_text.contains("conclusion"));
        Ok(())
    }

    #[test]
    fn test_strip_tool_call_text_removes_tool_descriptions() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk(
            "Some intro text\nreport({\"key\": \"value\"})\nMore text",
            StreamChunkKind::Text,
        );
        let call_id = Uuid::now_v7();
        builder.push_tool_call(
            "report".to_string(),
            call_id,
            Some(serde_json::json!({"key": "value"})),
            None,
        );
        builder.push_tool_result(
            "report".to_string(),
            call_id,
            Value::String("result".to_string()),
            true,
            None,
            None,
        );
        builder.push_chunk(
            "report.content(\"final output\")\nEnding",
            StreamChunkKind::Text,
        );

        let stream = builder.seal_coalesced();
        let display = stream.display_text();
        assert!(display.contains("Some intro text"), "should keep intro");
        assert!(display.contains("More text"), "should keep middle text");
        assert!(display.contains("Ending"), "should keep ending");
        assert!(!display.contains("report("), "should strip report(...)");
        assert!(
            !display.contains("report.content"),
            "should strip report.content(...)"
        );
        Ok(())
    }

    #[test]
    fn test_strip_tool_call_text_preserves_normal_text() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk(
            "Hello world\nThis is a report about things\nGoodbye",
            StreamChunkKind::Text,
        );

        let stream = builder.seal_coalesced();
        let display = stream.display_text();
        assert!(display.contains("Hello world"));
        assert!(display.contains("This is a report about things"));
        assert!(display.contains("Goodbye"));
        Ok(())
    }

    #[test]
    fn test_strip_tool_call_text_no_tool_calls_fallback() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk(
            "exec({\"code\": \"import { report } from 'hubris'; report()\"})",
            StreamChunkKind::Text,
        );

        let stream = builder.seal_coalesced();
        assert!(
            !stream.display_text().contains("exec("),
            "fallback should strip exec(...) even without tool call segments"
        );
        Ok(())
    }

    #[test]
    fn test_strip_tool_call_text_whitespace_and_json_variants() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        builder.push_chunk(
            "  exec({ \"code\": \"import { report } from 'hubris'; report({text: 'hello'})\" })\n\
             exec({\n  \"code\": \"const x = 1\"\n})\n\
             Some normal text here\n\
             Also normal",
            StreamChunkKind::Text,
        );

        let stream = builder.seal_coalesced();
        let display = stream.display_text();
        assert!(!display.contains("exec("), "should strip exec({{...)");
        assert!(
            display.contains("Some normal text here"),
            "should preserve normal text"
        );
        assert!(
            display.contains("Also normal"),
            "should preserve normal text"
        );
        assert!(
            display.contains("Also normal"),
            "should preserve trailing normal text"
        );
        Ok(())
    }

    #[test]
    fn test_seal_coalesced_closes_pending_tool_calls() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        let call_id_1 = Uuid::now_v7();
        let call_id_2 = Uuid::now_v7();

        builder.push_tool_call(
            "read_file".to_string(),
            call_id_1,
            Some(serde_json::to_value(PathParam { path: "/a" }).unwrap_or_default()),
            None,
        );
        builder.push_tool_result(
            "read_file".to_string(),
            call_id_1,
            Value::String("content A".to_string()),
            true,
            None,
            None,
        );
        builder.push_tool_call(
            "write_file".to_string(),
            call_id_2,
            Some(serde_json::to_value(PathParam { path: "/b" }).unwrap_or_default()),
            None,
        );

        assert!(builder.has_pending_tool_calls());

        let stream = builder.seal_coalesced();

        let tools: Vec<_> = stream.tool_segments().collect();
        assert_eq!(
            tools.len(),
            4,
            "should have 2 calls + 1 real result + 1 synthetic result"
        );

        let synthetic = tools.iter().find(|s| {
            matches!(s, StreamSegment::ToolResult { call_id, success, .. } if *call_id == call_id_2 && !success)
        });
        assert!(
            synthetic.is_some(),
            "should have synthetic result for pending call_id_2 with success=false"
        );
        let synth_data = synthetic
            .context("missing synthetic result")?
            .tool_result_data()
            .context("test precondition")?;
        assert_eq!(
            synth_data,
            &Value::String("[stream sealed before result arrived]".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_seal_coalesced_no_synthetic_when_all_resolved() -> Result<()> {
        let mut builder = LlmStreamBuilder::new();
        let call_id = Uuid::now_v7();
        builder.push_tool_call("exec".to_string(), call_id, Some(empty()), None);
        builder.push_tool_result(
            "exec".to_string(),
            call_id,
            Value::String("ok".to_string()),
            true,
            None,
            None,
        );

        assert!(!builder.has_pending_tool_calls());

        let stream = builder.seal_coalesced();
        let tools: Vec<_> = stream.tool_segments().collect();
        assert_eq!(
            tools.len(),
            2,
            "should have exactly call + result, no synthetic"
        );
        Ok(())
    }

    #[test]
    fn test_text_segment_returns_string_directly() -> Result<()> {
        let seg = StreamSegment::Text {
            text: "hello".to_string(),
            message_id: None,
        };
        assert_eq!(seg.text(), "hello");
        Ok(())
    }

    #[test]
    fn test_tool_call_params_structured() -> Result<()> {
        let seg = StreamSegment::ToolCall {
            tool_name: "exec".to_string(),
            call_id: Uuid::now_v7(),
            params: serde_json::json!({"code": "import { report } from 'hubris'; report({text: 'hi'})"}),
            agent_type: None,
            message_id: None,
        };
        let params = seg.tool_params().context("expected tool params")?;
        assert_eq!(
            params["code"],
            "import { report } from 'hubris'; report({text: 'hi'})"
        );
        Ok(())
    }

    #[test]
    fn test_tool_result_data_structured() -> Result<()> {
        let seg = StreamSegment::ToolResult {
            tool_name: "report_human".to_string(),
            call_id: Uuid::now_v7(),
            success: true,
            data: serde_json::json!({"ok": true, "data": "Hello!", "error": null}),
            duration_ms: None,
            agent_type: None,
            message_id: None,
        };
        let data = seg
            .tool_result_data()
            .context("expected tool result data")?;
        assert_eq!(data["ok"], true);
        assert_eq!(data["data"], "Hello!");
        Ok(())
    }
}
