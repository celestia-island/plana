use serde_json::Value;
use std::collections::HashSet;
use uuid::Uuid;

use tracing::warn;

use super::{StreamChunkKind, StreamSegment, rendering::LlmStream};

#[derive(Clone)]
pub struct LlmStreamBuilder {
    pub(crate) segments: Vec<StreamSegment>,
}

impl LlmStreamBuilder {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push_chunk(&mut self, chunk: &str, kind: StreamChunkKind) {
        self.push_chunk_with_id(chunk, kind, None)
    }

    fn push_chunk_with_id(&mut self, chunk: &str, kind: StreamChunkKind, message_id: Option<Uuid>) {
        if chunk.is_empty() {
            return;
        }
        if let Some(last) = self.segments.last_mut() {
            let compatible = matches!(
                (&*last, &kind),
                (StreamSegment::Text { .. }, StreamChunkKind::Text)
                    | (StreamSegment::Thinking { .. }, StreamChunkKind::Thinking)
                    | (
                        StreamSegment::DeepThinking { .. },
                        StreamChunkKind::DeepThinking
                    )
            );
            if compatible {
                let incoming_mid = message_id;
                match last {
                    StreamSegment::Text { text, message_id }
                    | StreamSegment::Thinking { text, message_id }
                    | StreamSegment::DeepThinking { text, message_id } => {
                        text.push_str(chunk);
                        if incoming_mid.is_some() {
                            *message_id = incoming_mid;
                        }
                    }
                    StreamSegment::ToolCall { .. } | StreamSegment::ToolResult { .. } => {
                        warn!("unexpected ToolCall/ToolResult segment in chunk push");
                    }
                }
                return;
            }
        }
        let seg = match kind {
            StreamChunkKind::Text => StreamSegment::Text {
                text: chunk.to_string(),
                message_id,
            },
            StreamChunkKind::Thinking => StreamSegment::Thinking {
                text: chunk.to_string(),
                message_id,
            },
            StreamChunkKind::DeepThinking => StreamSegment::DeepThinking {
                text: chunk.to_string(),
                message_id,
            },
        };
        self.segments.push(seg);
    }

    pub fn push_tool_call(
        &mut self,
        tool_name: String,
        call_id: Uuid,
        params: Option<Value>,
        agent_type: Option<String>,
    ) {
        self.strip_trailing_tool_text_for_call(&tool_name);
        self.segments.push(StreamSegment::ToolCall {
            tool_name,
            call_id,
            params: params.unwrap_or(Value::Null),
            agent_type,
            message_id: None,
        });
    }

    fn strip_trailing_tool_text_for_call(&mut self, tool_name: &str) {
        if self.segments.is_empty() {
            return;
        }
        let last_idx = self.segments.len() - 1;
        if let StreamSegment::Text { text, .. }
        | StreamSegment::Thinking { text, .. }
        | StreamSegment::DeepThinking { text, .. } = &mut self.segments[last_idx]
        {
            let pattern_prefix = format!("{}(", tool_name);
            let pattern_dot = format!("{}.content(", tool_name);
            let lines: Vec<&str> = text.lines().collect();
            let mut remove_count = 0;
            for line in lines.iter().rev() {
                let trimmed = line.trim_start();
                if trimmed.starts_with(&pattern_prefix)
                    || trimmed.starts_with(&pattern_dot)
                    || (trimmed.starts_with(tool_name)
                        && trimmed.contains('(')
                        && trimmed.contains('{'))
                {
                    remove_count += 1;
                } else {
                    break;
                }
            }
            if remove_count > 0 && remove_count >= lines.len() {
                text.clear();
            } else if remove_count > 0 {
                let keep = lines.len() - remove_count;
                *text = lines[..keep].join("\n");
            }
        }
    }

    pub fn push_tool_result(
        &mut self,
        tool_name: String,
        call_id: Uuid,
        data: Value,
        success: bool,
        duration_ms: Option<u64>,
        agent_type: Option<String>,
    ) {
        self.segments.push(StreamSegment::ToolResult {
            tool_name,
            call_id,
            success,
            data,
            duration_ms,
            agent_type,
            message_id: None,
        });
    }

    pub fn insert_tool_result_after_call(
        &mut self,
        call_id: Uuid,
        tool_name: String,
        data: Value,
        success: bool,
        duration_ms: Option<u64>,
        agent_type: Option<String>,
    ) {
        let result = StreamSegment::ToolResult {
            tool_name,
            call_id,
            success,
            data,
            duration_ms,
            agent_type,
            message_id: None,
        };
        let pos = self
            .segments
            .iter()
            .rposition(|s| {
                matches!(s, StreamSegment::ToolCall { .. }) && s.call_id() == Some(call_id)
            })
            .map(|i| i + 1)
            .unwrap_or(self.segments.len());
        self.segments.insert(pos, result);
    }

    pub fn coalesce_last_text_segments(&mut self) {
        let mut write_idx = 0;
        for read_idx in 0..self.segments.len() {
            if write_idx == 0 {
                write_idx = 1;
                continue;
            }
            let compatible = matches!(
                (&self.segments[write_idx - 1], &self.segments[read_idx]),
                (StreamSegment::Text { .. }, StreamSegment::Text { .. })
                    | (
                        StreamSegment::Thinking { .. },
                        StreamSegment::Thinking { .. }
                    )
                    | (
                        StreamSegment::DeepThinking { .. },
                        StreamSegment::DeepThinking { .. }
                    )
            );

            if compatible {
                let text_to_append = self.segments[read_idx].text_or_json();
                let dest = &mut self.segments[write_idx - 1];
                match dest {
                    StreamSegment::Text { text, .. }
                    | StreamSegment::Thinking { text, .. }
                    | StreamSegment::DeepThinking { text, .. } => {
                        text.push_str(&text_to_append);
                    }
                    _ => {}
                }
            } else {
                self.segments[write_idx] = self.segments[read_idx].clone();
                write_idx += 1;
            }
        }
        self.segments.truncate(write_idx);
    }

    pub fn as_str(&self) -> String {
        self.segments
            .iter()
            .filter(|s| s.is_text())
            .map(|s| s.text())
            .collect()
    }

    pub fn len(&self) -> usize {
        let mut total = 0;
        for seg in &self.segments {
            match seg {
                StreamSegment::Text { text, .. }
                | StreamSegment::Thinking { text, .. }
                | StreamSegment::DeepThinking { text, .. } => {
                    total += text.len();
                }
                StreamSegment::ToolCall { params, .. } => {
                    total += params.to_string().len();
                }
                StreamSegment::ToolResult { data, .. } => {
                    total += data.to_string().len();
                }
            }
        }
        total
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
            || self.segments.iter().all(|s| match s {
                StreamSegment::Text { text, .. }
                | StreamSegment::Thinking { text, .. }
                | StreamSegment::DeepThinking { text, .. } => text.is_empty(),
                StreamSegment::ToolCall { params, .. } => params.is_null(),
                StreamSegment::ToolResult { data, .. } => data.is_null(),
            })
    }

    pub fn segments(&self) -> &[StreamSegment] {
        &self.segments
    }

    pub fn segments_mut(&mut self) -> &mut [StreamSegment] {
        &mut self.segments
    }

    pub fn seal(self) -> LlmStream {
        LlmStream {
            segments: self.segments,
        }
    }

    pub fn seal_coalesced(mut self) -> LlmStream {
        self.coalesce_last_text_segments();
        self.close_pending_tool_calls();
        if let Err(e) = self.strip_tool_call_text_from_text_segments() {
            warn!(error = %e, "failed to strip tool call text from text segments");
        }
        self.seal()
    }

    pub fn seal_coalesced_deferred(mut self) -> LlmStream {
        self.coalesce_last_text_segments();
        if let Err(e) = self.strip_tool_call_text_from_text_segments() {
            warn!(error = %e, "failed to strip tool call text from text segments");
        }
        self.seal()
    }

    fn close_pending_tool_calls(&mut self) {
        let mut closed_ids: HashSet<Uuid> = HashSet::new();
        for seg in &self.segments {
            if let StreamSegment::ToolResult { call_id, .. } = seg {
                closed_ids.insert(*call_id);
            }
        }
        let mut synthetics: Vec<StreamSegment> = Vec::new();
        for seg in &self.segments {
            if let StreamSegment::ToolCall {
                call_id,
                tool_name,
                agent_type,
                ..
            } = seg
                && !closed_ids.contains(call_id)
            {
                synthetics.push(StreamSegment::ToolResult {
                    tool_name: tool_name.clone(),
                    call_id: *call_id,
                    success: false,
                    data: Value::String("[stream sealed before result arrived]".to_string()),
                    duration_ms: None,
                    agent_type: agent_type.clone(),
                    message_id: None,
                });
                closed_ids.insert(*call_id);
            }
        }
        self.segments.extend(synthetics);
    }

    fn strip_tool_call_text_from_text_segments(&mut self) -> Result<(), regex::Error> {
        let tool_names: Vec<String> = {
            let from_segments: Vec<String> = self
                .segments
                .iter()
                .filter_map(|seg| match seg {
                    StreamSegment::ToolCall { tool_name, .. } => Some(tool_name.clone()),
                    _ => None,
                })
                .collect();

            if from_segments.is_empty() {
                vec!["exec".to_string()]
            } else {
                from_segments
            }
        };

        let pattern = {
            let alternation = tool_names
                .iter()
                .map(|n| regex::escape(n))
                .collect::<Vec<_>>()
                .join("|");
            regex::Regex::new(&format!(
                r"(?m)^\s*(?:{0})\s*(?:\(|\.content\(|\(\s*\{{)",
                alternation
            ))?
        };

        for seg in &mut self.segments {
            let text = match seg {
                StreamSegment::Text { text, .. } => text,
                _ => continue,
            };
            if text.trim().is_empty() {
                continue;
            }
            let mut cleaned_lines = Vec::new();
            for line in text.lines() {
                if !pattern.is_match(line) {
                    cleaned_lines.push(line);
                }
            }
            *text = cleaned_lines.join("\n");
        }
        Ok(())
    }

    pub fn has_pending_tool_calls(&self) -> bool {
        let mut call_ids: HashSet<Uuid> = HashSet::new();
        for seg in &self.segments {
            match seg {
                StreamSegment::ToolCall { call_id, .. } => {
                    call_ids.insert(*call_id);
                }
                StreamSegment::ToolResult { call_id, .. } => {
                    call_ids.remove(call_id);
                }
                _ => {}
            }
        }
        !call_ids.is_empty()
    }
}

impl Default for LlmStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}
